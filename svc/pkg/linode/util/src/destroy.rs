use rivet_operation::prelude::*;

use crate::Client;

pub async fn delete_ssh_key(client: &Client, ssh_key_id: i64) -> GlobalResult<()> {
	tracing::info!("deleting linode ssh key");

	client
		.delete(&format!("/profile/sshkeys/{ssh_key_id}"))
		.await
}

pub async fn delete_instance(client: &Client, linode_id: i64) -> GlobalResult<()> {
	tracing::info!("deleting linode instance");

	client
		.delete(&format!("linode/instances/{linode_id}"))
		.await
}

pub async fn delete_firewall(client: &Client, firewall_id: i64) -> GlobalResult<()> {
	tracing::info!("deleting firewall");

	client
		.delete(&format!("networking/firewalls/{firewall_id}"))
		.await
}
