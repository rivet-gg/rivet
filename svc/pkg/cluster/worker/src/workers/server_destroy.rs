use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[derive(sqlx::FromRow)]
struct Server {
	datacenter_id: Uuid,
	provider_server_id: Option<String>,
	cloud_destroy_ts: Option<i64>,
}

#[worker(name = "cluster-server-destroy")]
async fn worker(ctx: &OperationContext<cluster::msg::server_destroy::Message>) -> GlobalResult<()> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	let server = sql_fetch_one!(
		[ctx, Server]
		"
		SELECT
			datacenter_id, provider_server_id, cloud_destroy_ts
		FROM db_cluster.servers
		WHERE
			server_id = $1
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
	}).await?;
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

	Ok(())
}
