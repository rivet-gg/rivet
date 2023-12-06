use proto::backend::pkg::*;
use reqwest::header;
use rivet_operation::prelude::*;

mod api;
use api::*;

#[derive(sqlx::FromRow)]
struct LinodeData {
	provider_server_id: String,
	ssh_key_id: i64,
	firewall_id: i64,
}

#[operation(name = "linode-server-destroy")]
pub async fn handle(
	ctx: OperationContext<linode::server_destroy::Request>,
) -> GlobalResult<linode::server_destroy::Response> {
	let server_id = unwrap!(ctx.server_id).as_uuid();

	let data = sql_fetch_one!(
		[ctx, LinodeData]
		"
		SELECT provider_server_id, ssh_key_id, firewall_id
		FROM db_cluster.servers AS s
		INNER JOIN db_cluster.linode_misc AS l
		ON s.server_id = l.server_id
		WHERE s.server_id = $1
		",
		server_id,
	)
	.await?;

	// Build HTTP client
	let api_token = util::env::read_secret(&["linode", "token"]).await?;
	let auth = format!("Bearer {}", api_token,);
	let mut headers = header::HeaderMap::new();
	headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&auth)?);
	let client = reqwest::Client::builder()
		.default_headers(headers)
		.build()?;

	delete_instance(&client, &data.provider_server_id).await?;
	delete_ssh_key(&client, data.ssh_key_id).await?;
	delete_firewall(&client, data.firewall_id).await?;

	Ok(linode::server_destroy::Response {})
}
