use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker(name = "cluster-server-provision")]
async fn worker(ctx: &OperationContext<cluster::msg::server_provision::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let server_id = unwrap!(ctx.server_id).as_uuid();
	let provider = unwrap!(backend::cluster::Provider::from_i32(ctx.provider));
	
	// NOTE: sql record already exists before this worker is called
	let server_row = sql_fetch_one!(
		[ctx, (Option<String>,), &crdb]
		"
		SELECT
			provider_server_id
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;

	if let (Some(provider_server_id),) = server_row {
		tracing::error!(?server_id, ?provider_server_id, "server is already provisioned");
		return Ok(());
	};

	let provider_server_id = match provider {
		backend::cluster::Provider::Linode => {
			let res = op!([ctx] linode_server_provision {
				cluster_id: ctx.cluster_id,
				datacenter_id: ctx.datacenter_id,
				server_id: ctx.server_id,
			})
			.await;

			if let Err(err) = res {
				tracing::error!(?err, "failed to provision linode server, destroying gracefully");

				// TODO: 
				msg!([ctx] cluster::msg::server_destroy() {

				}).await?;
			}

			res.provider_server_id.clone()
		}
	};

	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.servers
		SET provider_server_id = $2
		WHERE server_id = $1
		",
		server_id,
		provider_server_id,
	)
	.await?;

	Ok(())
}
