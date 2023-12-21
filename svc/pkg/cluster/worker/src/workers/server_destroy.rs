use chirp_worker::prelude::*;
use cloudflare::{endpoints as cf, framework as cf_framework, framework::async_api::ApiClient};
use proto::backend::{self, pkg::*};

#[derive(sqlx::FromRow)]
struct Server {
	datacenter_id: Uuid,
	provider_server_id: Option<String>,
	dns_record_id: Option<String>,
}

#[worker(name = "cluster-server-destroy")]
async fn worker(ctx: &OperationContext<cluster::msg::server_destroy::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	let server = sql_fetch_one!(
		[ctx, Server, &crdb]
		"
		SELECT
			datacenter_id, provider_server_id, dns_record_id
		FROM db_cluster.servers as s
		LEFT JOIN db_cluster.cloudflare_misc as cf
		ON s.server_id = cf.server_id
		WHERE s.server_id = $1
		",
		&server_id,
		util::timestamp::now(),
	)
	.await?;

	// We wait for the install process to complete to make sure the destroy is clean
	if server.provider_server_id.is_none() {
		retry_bail!("server install process is not complete, retrying");
	}

	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![server.datacenter_id.into()],
	})
	.await?;
	let datacenter = unwrap!(datacenter_res.datacenters.first());

	let provider = unwrap!(backend::cluster::Provider::from_i32(datacenter.provider));

	match provider {
		backend::cluster::Provider::Linode => {
			tracing::info!(?server_id, "destroying linode server");

			op!([ctx] linode_server_destroy {
				server_id: ctx.server_id,
			})
			.await?;
		}
	}

	if let Some(dns_record_id) = server.dns_record_id {
		tracing::info!(?server_id, "deleting dns record");
		delete_dns_record(ctx, &crdb, server_id, &dns_record_id).await?;
	}

	msg!([ctx] cluster::msg::server_destroy_complete(server_id) {
		server_id: ctx.server_id,
	})
	.await?;

	Ok(())
}

async fn delete_dns_record(
	ctx: &OperationContext<cluster::msg::server_destroy::Message>,
	crdb: &CrdbPool,
	server_id: Uuid,
	dns_record_id: &str,
) -> GlobalResult<()> {
	let cf_token = util::env::read_secret(&["cloudflare", "terraform", "auth_token"]).await?;
	let zone_id = unwrap!(util::env::cloudflare::zone::job::id(), "dns not configured");

	// Create cloudflare HTTP client
	let client = cf_framework::async_api::Client::new(
		cf_framework::auth::Credentials::UserAuthToken { token: cf_token },
		Default::default(),
		cf_framework::Environment::Production,
	)
	.map_err(crate::CloudflareError::from)?;

	client
		.request(&cf::dns::DeleteDnsRecord {
			zone_identifier: zone_id,
			identifier: dns_record_id,
		})
		.await?;

	// Remove record
	sql_execute!(
		[ctx, &crdb]
		"
		DELETE FROM db_cluster.cloudflare_misc
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;

	Ok(())
}
