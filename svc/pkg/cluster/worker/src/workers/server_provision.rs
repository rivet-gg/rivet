use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

struct ProvisionResponse {
	provider_server_id: String,
	vlan_ip: String,
	public_ip: String,
}

#[worker(name = "cluster-server-provision")]
async fn worker(ctx: &OperationContext<cluster::msg::server_provision::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	
	let cluster_id = unwrap!(ctx.cluster_id);
	let server_id = unwrap!(ctx.server_id).as_uuid();
	let provider = unwrap!(backend::cluster::Provider::from_i32(ctx.provider));
	
	// Check if server is already provisioned
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
	
	let cluster_res = op!([ctx] cluster_get {
		cluster_ids: vec![cluster_id],
	}).await?;
	let cluster = unwrap!(cluster_res.clusters.first());
	let datacenter = unwrap!(cluster.datacenters.iter().find(|dc| dc.datacenter_id == ctx.datacenter_id));

	let provision_res = match provider {
		backend::cluster::Provider::Linode => {
			let mut hardware_list = datacenter.hardware.iter();

			// Iterate through list of hardware and attempt to schedule a server
			loop {
				// List exhausted
				let Some(hardware) = hardware_list.next() else {
					break None;
				};

				let res = op!([ctx] linode_server_provision {
					server_id: ctx.server_id,
					provider_datacenter_id: datacenter.provider_datacenter_id.clone(),
					hardware: Some(hardware.clone()),
					pool_type: ctx.pool_type,
				})
				.await;
	
				match res {
					Ok(res) => {
						break Some(ProvisionResponse {
							provider_server_id: res.provider_server_id.clone(),
							vlan_ip: res.vlan_ip.clone(),
							public_ip: res.public_ip.clone(),
						})
					},
					Err(err) => {
						tracing::warn!(?err, "failed to provision linode server, destroying gracefully");
			
						// TODO: 
						// op!([ctx] linode_server_destroy {
						// })
						// .await?;
						todo!();
					}
				}
			}
		}
	};

	// All attempts to provision failed
	let Some(provision_res) = provision_res else {
		tracing::info!(?server_id, "failed to provision server");
		bail!("failed to provision server");
	};

	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.servers
		SET
			provider_server_id = $2,
			vlan_ip = $3,
			public_ip = $4
		WHERE server_id = $1
		",
		server_id,
		provision_res.provider_server_id,
		provision_res.vlan_ip,
		provision_res.public_ip,
	)
	.await?;

	// TODO:
	// msg!([ctx] cluster::msg::server_install(cluster_id, datacenter_id, server_id) {
	// 	server_id: ctx.server_id,
	// })
	// .await?;

	Ok(())
}
