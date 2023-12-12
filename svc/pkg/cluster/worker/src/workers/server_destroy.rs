use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker(name = "cluster-server-destroy")]
async fn worker(ctx: &OperationContext<cluster::msg::server_destroy::Message>) -> GlobalResult<()> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	let row = sql_fetch_optional!(
		[ctx, (Uuid,)]
		"
		UPDATE db_cluster.servers
		SET cloud_destroy_ts = $2
		WHERE
			server_id = $1 AND
			cloud_destroy_ts IS NULL
		RETURNING datacenter_id
		",
		&server_id,
		util::timestamp::now(),
	)
	.await?;
	let Some((datacenter_id,)) = row else {
		tracing::warn!("trying to delete server that was already deleted");
		return Ok(());
	};

	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id.into()],
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
