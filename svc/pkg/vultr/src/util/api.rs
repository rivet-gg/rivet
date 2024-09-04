use std::{net::Ipv4Addr, str, time::Duration};

use chirp_workflow::prelude::*;
use serde::Deserialize;
use serde_json::json;
use ssh_key::PrivateKey;

use crate::{
	util::client::Client,
};

#[derive(Deserialize)]
struct CreateSshKeyResponse {
	ssh_key: SshKey,
}

#[derive(Deserialize)]
pub struct SshKey {
	pub id: Uuid,
}

pub async fn create_ssh_key(client: &Client, label: &str, is_test: bool) -> GlobalResult<SshKey> {
	tracing::info!("creating vultr ssh key");

	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let private_key = PrivateKey::from_openssh(private_key_openssh.as_bytes())?;

	// Extract the public key
	let public_key = private_key.public_key().to_string();

	// HACK: We use this when cleaning up tests; we check if the label has `test-` in it
	let label = if is_test {
		format!("test-{label}")
	} else {
		label.to_string()
	};

	let res = client
		.post::<CreateSshKeyResponse>(
			"/ssh-keys",
			json!({
				"name": label,
				"ssh_key": public_key,
			}),
		)
		.await?;

	Ok(res.ssh_key)
}

#[derive(Deserialize)]
struct CreateInstanceResponse {
	instance: Instance,
}

#[derive(Deserialize)]
pub struct Instance {
	pub id: Uuid,
	pub status: String,
	pub main_ip: Ipv4Addr,
}

pub async fn create_instance(
	client: &Client,
	name: &str,
	datacenter: &str,
	hardware: &str,
	tags: &[String],
	ssh_key_id: &str,
) -> GlobalResult<Instance> {
	tracing::info!("creating vultr instance");

	let res = client
		.post::<CreateInstanceResponse>(
			"/instances",
			json!({
				"label": name,
				// Debian 12
				// https://www.vultr.com/api/#tag/s3/operation/list-object-storage-clusters
				"os_id": 2136,
				"region": datacenter,
				"plan": hardware,
				"ssh_key_id": ssh_key_id,
				"tags": tags,
				"enable_ipv6": false,
				"disable_public_ipv4": false,
				"enable_vpc2": true,
				"backups": false,
			}),
		)
		.await?;

	Ok(res.instance)
}

#[derive(Deserialize)]
struct CreateFirewallGroupResponse {
	firewall_group: FirewallGroup,
}

#[derive(Deserialize)]
pub struct FirewallGroup {
	pub id: Uuid,
}

pub async fn create_firewall_group(client: &Client, name: &str) -> GlobalResult<FirewallGroup> {
	tracing::info!("creating firewall group");

	let res = client
		.post::<CreateFirewallGroupResponse>(
			"/firewalls",
			json!({
				"description": name,
			}),
		)
		.await?;

	Ok(res.firewall_group)
}

// NOTE: Vultr firewalls drop all by default
// https://docs.vultr.com/vultr-firewall#:~:text=What%20is%20the%20default%20policy%20of%20Vultr%20Firewall%3F
pub async fn create_firewall_rule(
	client: &Client,
	firewall_group_id: Uuid,
	label: &str,
	ip_type: &str,
	protocol: util::net::Protocol,
	port: util::net::Port,
	subnet: String,
	subnet_size: u16,
) -> GlobalResult<()> {
	tracing::info!("creating firewall rule");

	client
		.post(
			&format!("/firewalls/{firewall_group_id}/rules"),
			json!({
				"ip_type": ip_type,
				"notes": label,
				"protocol": protocol.as_uppercase(),
				"port": match port {
					util::net::Port::Single(p) => p.to_string(),
					util::net::Port::Range(s, e) => format!("{s}:{e}"),
				},
				"subnet": subnet,
				"subnet_size": subnet_size,
			}),
		)
		.await
}

#[derive(Deserialize)]
struct GetInstanceResponse {
	instance: Instance,
}

/// Polls vultr API until an instance is available.
pub async fn wait_instance_ready(client: &Client, instance_id: Uuid) -> GlobalResult<Ipv4Addr> {
	tracing::info!("waiting for instance to be ready");

	loop {
		let res = client
			.get::<GetInstanceResponse>(&format!("/instances/{instance_id}"))
			.await?;

		// Check if ready
		if res.instance.status.as_str() == "active" && res.instance.main_ip != Ipv4Addr::new(0, 0, 0, 0) {
			return Ok(res.instance.main_ip);
		}

		tokio::time::sleep(Duration::from_secs(1)).await;
	}
}

pub async fn delete_ssh_key(client: &Client, ssh_key_id: Uuid) -> GlobalResult<()> {
	tracing::info!("deleting linode ssh key");

	client.delete(&format!("/ssh-keys/{ssh_key_id}")).await
}

pub async fn delete_instance(client: &Client, instance_id: Uuid) -> GlobalResult<()> {
	tracing::info!(?instance_id, "deleting linode instance");

	client.delete(&format!("/instances/{instance_id}")).await
}

pub async fn delete_firewall_group(client: &Client, firewall_group_id: Uuid) -> GlobalResult<()> {
	tracing::info!("deleting firewall group");

	client
		.delete(&format!("/firewalls/{firewall_group_id}"))
		.await
}

#[derive(Deserialize)]
pub struct ListPlansResponse {
	pub plans: Vec<Plan>,
}

#[derive(Deserialize)]
pub struct Plan {
	pub id: String,
	pub ram: u64,  // MB
	pub disk: u64, // GB
	pub vcpu_count: u64,
	pub bandwidth: u64, // GB
}

impl From<Plan> for crate::types::Plan {
	fn from(value: Plan) -> Self {
		crate::types::Plan {
			hardware_id: value.id,
			memory: value.ram,
			disk: value.disk,
			vcpus: value.vcpu_count,
			transfer: value.bandwidth,
		}
	}
}

pub async fn list_plans(client: &Client) -> GlobalResult<Vec<Plan>> {
	tracing::info!("listing plans");

	let res = client.get::<ListPlansResponse>("/plans").await?;

	Ok(res.plans)
}
