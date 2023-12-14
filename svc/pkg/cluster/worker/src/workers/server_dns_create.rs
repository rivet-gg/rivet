use std::net::Ipv4Addr;

use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};
use cloudflare::{endpoints as cf, framework as cf_framework, framework::async_api::ApiClient};

#[derive(sqlx::FromRow)]
struct Server {
	datacenter_id: Uuid,
	pool_type: i64,
	public_ip: String,
	cloud_destroy_ts: Option<i64>,
}

#[worker(name = "cluster-server-dns-create")]
async fn worker(ctx: &OperationContext<cluster::msg::server_dns_create::Message>) -> GlobalResult<()> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();
	
	let server = sql_fetch_one!(
		[ctx, Server]
		"
		SELECT
			datacenter_id, pool_type, public_ip, cloud_destroy_ts
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;

	if server.cloud_destroy_ts.is_some() {
		tracing::info!("server marked for deletion, not creating dns record");
		return Ok(());
	}
	
	// NOTE: The only pool type that needs DNS records should be GG
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(server.pool_type as i32));
	let cf_token = util::env::read_secret(&["cloudflare", "terraform", "auth_token"]).await?;
	let zone_id = unwrap!(util::env::cloudflare::zone::job::id(), "dns not configured");
	let record_name = format!("*.lobby.{}.{}", server.datacenter_id, unwrap!(util::env::domain_job()));
	let public_ip = server.public_ip.as_str().parse::<Ipv4Addr>()?;

	// Create cloudflare HTTP client
	let client = cf_framework::async_api::Client::new(
		cf_framework::auth::Credentials::UserAuthToken { token: cf_token },
		Default::default(),
		cf_framework::Environment::Production,
	)
	.map_err(crate::CloudflareError::from)?;

	let create_record_res = client
		.request(&cf::dns::CreateDnsRecord {
			zone_identifier: zone_id,
			params: cf::dns::CreateDnsRecordParams {
				name: &record_name,
				content: cf::dns::DnsContent::A { content: public_ip },
				proxied: Some(false),
				ttl: Some(60),
				priority: None,
			},
		})
		.await?;

	// Save record id for deletion
	let record_id = create_record_res.result.id;
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.cloudflare_misc (
			server_id,
			dns_record_id
		)
		VALUES ($1, $2)
		",
		server_id,
		record_id
	)
	.await?;

	Ok(())
}
