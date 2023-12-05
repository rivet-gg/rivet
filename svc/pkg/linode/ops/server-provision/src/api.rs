use std::{net::Ipv4Addr, fmt, str};

use openssl::{pkey::PKey, rsa::Rsa};
use rand::{distributions::Alphanumeric, Rng};
use rivet_operation::prelude::*;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::json;

use crate::ServerCtx;

#[derive(Deserialize)]
struct ApiErrorResponse {
	errors: Vec<ApiError>,
}

impl fmt::Display for ApiErrorResponse {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for error in &self.errors {
			writeln!(f, "{:?}: {}", error.field, error.reason)?;
		}

		Ok(())
	}
}

#[derive(Deserialize)]
struct ApiError {
	field: String,
	reason: String,
}

pub async fn create_ssh_key(client: &reqwest::Client, server: &ServerCtx) -> GlobalResult<String> {
	tracing::info!("creating ssh key");
	
	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let private_key = Rsa::private_key_from_pem(private_key_openssh.as_bytes())?;

	// Extract the public key
	let public_key = PKey::from_rsa(private_key)?;
	let public_key_pem = public_key.public_key_to_pem()?;
	let public_key_str = str::from_utf8(public_key_pem.as_slice())?.trim();

	let res = client
		.post("https://api.linode.com/v4/profile/sshkeys")
		.header("content-type", "application/json")
		.json(&json!({
			"label": server.name,
			"ssh_key": public_key_str,
		}))
		.send()
		.await?;
	handle_response(res).await?;

	Ok(public_key_str.to_string())
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

pub async fn create_firewall(
	client: &reqwest::Client,
	linode_id: u64,
	server: &ServerCtx,
) -> GlobalResult<()> {
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

				"ipv4": rule.inbound_ipv4_cidr,
				"ipv6": rule.inbound_ipv6_cidr,
			})
		})
		.collect::<Vec<_>>();

	let res = client
		.post("https://api.linode.com/v4/networking/firewalls")
		.header("content-type", "application/json")
		.json(&json!({
			"label": server.name,
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

	handle_response(res).await
}

#[derive(Deserialize)]
pub struct GetPublicIpResponse {
	ipv4: LinodeIpv4,
}

#[derive(Deserialize)]
pub struct LinodeIpv4 {
	public: LinodeIpv4Config,
}

#[derive(Deserialize)]
pub struct LinodeIpv4Config {
	address: Ipv4Addr,
}

pub async fn get_public_ip(
	client: &reqwest::Client,
	linode_id: u64,
) -> GlobalResult<Ipv4Addr> {
	tracing::info!("getting ip");
	
	let res = client
		.get(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}/ips"
		))
		.send()
		.await?;

	let res = parse_response::<GetPublicIpResponse>(res).await?;

	Ok(res.ipv4.public.address)
}

async fn handle_response(res: reqwest::Response) -> GlobalResult<()> {
	if !res.status().is_success() {
		bail_with!(ERROR, error = res.json::<ApiErrorResponse>().await?);
	}

	Ok(())
}

async fn parse_response<T: DeserializeOwned>(res: reqwest::Response) -> GlobalResult<T> {
	if !res.status().is_success() {
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
