use std::fmt;

use rivet_operation::prelude::*;
use serde::Deserialize;

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

pub async fn delete_ssh_key(client: &reqwest::Client, ssh_key_id: i64) -> GlobalResult<()> {
	tracing::info!("deleting linode ssh key");

	let res = client
		.delete(format!(
			"https://api.linode.com/v4/profile/sshkeys/{ssh_key_id}"
		))
		.send()
		.await?;

	handle_response(res).await?;

	Ok(())
}

pub async fn delete_instance(client: &reqwest::Client, linode_id: &str) -> GlobalResult<()> {
	tracing::info!("deleting linode instance");

	let res = client
		.delete(format!(
			"https://api.linode.com/v4/linode/instances/{linode_id}"
		))
		.send()
		.await?;

	handle_response(res).await?;

	Ok(())
}

pub async fn delete_firewall(client: &reqwest::Client, firewall_id: i64) -> GlobalResult<()> {
	tracing::info!("deleting firewall");

	let res = client
		.delete(format!(
			"https://api.linode.com/v4/networking/firewalls/{firewall_id}"
		))
		.send()
		.await?;

	handle_response(res).await?;

	Ok(())
}

async fn handle_response(res: reqwest::Response) -> GlobalResult<()> {
	if !res.status().is_success() {
		// Resource does not exist to be deleted, not an error
		if res.status() == reqwest::StatusCode::NOT_FOUND {
			tracing::info!("resource doesn't exist");
			return Ok(());
		}

		bail_with!(ERROR, error = res.json::<ApiErrorResponse>().await?);
	}

	Ok(())
}
