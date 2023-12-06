use std::{fmt, net::Ipv4Addr, str, time::Duration};

use rand::{distributions::Alphanumeric, Rng};
use rivet_operation::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;
use ssh_key::PrivateKey;

use crate::ServerCtx;

#[derive(Deserialize)]
struct ApiErrorResponse {
	errors: Vec<ApiError>,
}

impl fmt::Display for ApiErrorResponse {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for error in &self.errors {
			if let Some(field) = &error.field {
				write!(f, "{:?}: ", field)?;
			}

			writeln!(f, "{}", error.reason)?;
		}

		Ok(())
	}
}

#[derive(Deserialize)]
struct ApiError {
	field: Option<String>,
	reason: String,
}

#[derive(Deserialize)]
struct CreateSshKeyResponse {
	id: u64,
}

pub struct SshKeyResponse {
	pub id: u64,
	pub public_key: String,
}

pub async fn create_ssh_key(
	client: &reqwest::Client,
	server: &ServerCtx,
) -> GlobalResult<SshKeyResponse> {
	tracing::info!("creating linode ssh key");

	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let private_key = PrivateKey::from_openssh(private_key_openssh.as_bytes())?;

	// Extract the public key
	let public_key = private_key.public_key().to_string();

	let res = client
		.post("https://api.linode.com/v4/profile/sshkeys")
		.header("content-type", "application/json")
		.json(&json!({
			"label": server.name,
			"ssh_key": public_key,
		}))
		.send()
		.await?;
	let res = parse_response::<CreateSshKeyResponse>(res).await?;

	Ok(SshKeyResponse {
		id: res.id,
		public_key,
	})
}

#[derive(Deserialize)]
pub struct CreateInstanceResponse {
	pub id: u64,
	pub specs: InstanceSpec,
}

#[derive(Deserialize)]
pub struct InstanceSpec {
	pub disk: u64,
}

pub async fn create_instance(
	client: &reqwest::Client,
	ns: &str,
	ssh_key: &str,
	server: &ServerCtx,
) -> GlobalResult<CreateInstanceResponse> {
	tracing::info!("creating linode instance");

	let res = client
		.post("https://api.linode.com/v4/linode/instances")
		.header("content-type", "application/json")
		.json(&json!({
			"label": server.name,
			"group": ns,
			"region": server.provider_datacenter_id,
			"type": server.provider_hardware,
			"authorized_keys": vec![ssh_key],
			"tags": server.tags,
			"private_ip": true,
			"backups_enabled": false,
		}))
		.send()
		.await?;

	parse_response(res).await
}

#[derive(Deserialize)]
pub struct CreateDiskResponse {
	pub id: u64,
}

pub struct CreateDisksResponse {
	pub boot_id: u64,
	pub swap_id: u64,
}

pub async fn create_disks(
	client: &reqwest::Client,
	ssh_key: &str,
	linode_id: u64,
	server_disk_size: u64,
) -> GlobalResult<CreateDisksResponse> {
	tracing::info!("creating boot disk");

	let boot_disk_res = client
		.post(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/disks"
		))
		.header("content-type", "application/json")
		.json(&json!({
			"label": "boot",
			"size": server_disk_size - 512,
			"authorized_keys": vec![ssh_key],
			"root_pass": generate_password(16),
			"image": "linode/debian11",
		}))
		.send()
		.await?;
	let boot_disk_res = parse_response::<CreateDiskResponse>(boot_disk_res).await?;

	wait_disk_ready(client, linode_id, boot_disk_res.id).await?;

	tracing::info!("creating swap disk");

	let swap_disk_res = client
		.post(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/disks"
		))
		.header("content-type", "application/json")
		.json(&json!({
			"label": "swap",
			"size": 512,
			"filesystem": "swap",
		}))
		.send()
		.await?;
	let swap_disk_res = parse_response::<CreateDiskResponse>(swap_disk_res).await?;

	Ok(CreateDisksResponse {
		boot_id: boot_disk_res.id,
		swap_id: swap_disk_res.id,
	})
}

pub async fn create_instance_config(
	client: &reqwest::Client,
	ns: &str,
	linode_id: u64,
	disks: &CreateDisksResponse,
	server: &ServerCtx,
) -> GlobalResult<()> {
	tracing::info!("creating instance config");

	let region_vlan = util::net::region::vlan_ip_net();
	let ipam_address = format!("{}/{}", server.vlan_ip, region_vlan.prefix_len());

	let res = client
		.post(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/configs"
		))
		.header("content-type", "application/json")
		.json(&json!({
			"label": "boot_config",
			"booted": true,
			"kernel": "linode/latest-64bit",
			"root_device": "/dev/sda",
			"devices": {
				"sda": {
					"disk_id": disks.boot_id,
				},
				"sdb": {
					"disk_id": disks.swap_id,
				},
			},
			"interfaces": [
				{
					"purpose": "public",
				},
				{
					"purpose": "vlan",
					"label": format!("{ns}-vlan"),
					"ipam_address": ipam_address,
				},
			],
		}))
		.send()
		.await?;

	handle_response(res).await
}

#[derive(Deserialize)]
pub struct CreateFirewallResponse {
	pub id: u64,
}

pub async fn create_firewall(
	client: &reqwest::Client,
	ns: &str,
	linode_id: u64,
	server: &ServerCtx,
) -> GlobalResult<CreateFirewallResponse> {
	tracing::info!("creating firewall");

	let firewall_inbound = server
		.firewall_inbound
		.iter()
		.map(|rule| {
			json!({
				"label": rule.label,
				"action": "ACCEPT",
				"protocol": rule.protocol.to_uppercase(),
				"ports": rule.ports,
				"addresses": {
					"ipv4": rule.inbound_ipv4_cidr,
					"ipv6": rule.inbound_ipv6_cidr,
				},

			})
		})
		.collect::<Vec<_>>();

	let res = client
		.post("https://api.linode.com/v4/networking/firewalls")
		.header("content-type", "application/json")
		.json(&json!({
			// Label doesn't matter
			"label": format!("{ns}-{}", generate_password(16)),
			"rules": {
				"inbound": firewall_inbound,
				"inbound_policy": "DROP",
				"outbound_policy": "ACCEPT",
			},
			"devices": {
				"linodes": [linode_id],
			},
		}))
		.send()
		.await?;

	parse_response(res).await
}

pub async fn boot_instance(client: &reqwest::Client, linode_id: u64) -> GlobalResult<()> {
	tracing::info!("booting instance");

	let res = client
		.post(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/boot"
		))
		.send()
		.await?;

	handle_response(res).await?;

	Ok(())
}

#[derive(Deserialize)]
pub struct LinodeInstanceResponse {
	status: String,
}

// Helpful: https://www.linode.com/community/questions/11588/linodeerrorsapierror-400-linode-busy
/// Polls linode API until an instance is available.
pub async fn wait_instance_ready(client: &reqwest::Client, linode_id: u64) -> GlobalResult<()> {
	tracing::info!("waiting for instance to be ready");

	loop {
		let res = client
			.get(format!(
				"https://api.linode.com/v4/linode/instances/{linode_id}"
			))
			.send()
			.await?;
		let res = parse_response::<LinodeInstanceResponse>(res).await?;

		// Check if ready
		match res.status.as_str() {
			"booting" | "rebooting" | "shutting_down" | "provisioning" | "deleting"
			| "migrating" | "rebuilding" | "cloning" | "restoring" => {}
			_ => break,
		}

		tokio::time::sleep(Duration::from_secs(1)).await;
	}

	Ok(())
}

#[derive(Deserialize)]
pub struct LinodeDiskResponse {
	status: String,
}

/// Polls linode API until a linode disk is available.
pub async fn wait_disk_ready(
	client: &reqwest::Client,
	linode_id: u64,
	disk_id: u64,
) -> GlobalResult<()> {
	tracing::info!("waiting for linode disk to be ready");

	loop {
		let res = client
			.get(format!(
				"https://api.linode.com/v4/linode/instances/{linode_id}/disks/{disk_id}"
			))
			.send()
			.await?;

		// Manually handle the disk showing up as not found yet
		if res.status() == reqwest::StatusCode::NOT_FOUND {
			tracing::info!("disk not found yet");
		} else {
			let res = parse_response::<LinodeDiskResponse>(res).await?;

			// Check if ready
			match res.status.as_str() {
				"not ready" => {}
				_ => break,
			}
		}

		tokio::time::sleep(Duration::from_secs(1)).await;
	}

	Ok(())
}

#[derive(Deserialize)]
pub struct GetPublicIpResponse {
	ipv4: LinodeIpv4,
}

#[derive(Deserialize)]
pub struct LinodeIpv4 {
	public: Vec<LinodeIpv4Config>,
}

#[derive(Deserialize)]
pub struct LinodeIpv4Config {
	address: Ipv4Addr,
}

pub async fn get_public_ip(client: &reqwest::Client, linode_id: u64) -> GlobalResult<Ipv4Addr> {
	tracing::info!("getting ip");

	let res = client
		.get(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/ips"
		))
		.send()
		.await?;

	let res = parse_response::<GetPublicIpResponse>(res).await?;
	let public = unwrap!(res.ipv4.public.first());

	Ok(public.address)
}

async fn handle_response(res: reqwest::Response) -> GlobalResult<()> {
	if !res.status().is_success() {
		bail_with!(ERROR, error = res.json::<ApiErrorResponse>().await?);
	}

	Ok(())
}

async fn parse_response<T: DeserializeOwned>(res: reqwest::Response) -> GlobalResult<T> {
	if !res.status().is_success() {
		tracing::info!(status=?res.status(), "api request failed");
		bail_with!(ERROR, error = res.json::<ApiErrorResponse>().await?);
	}

	let res = res.json::<T>().await?;
	Ok(res)
}

/// Generates a random string for a secret.
fn generate_password(length: usize) -> String {
	rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(length)
		.map(char::from)
		.collect()
}
