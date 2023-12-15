use chirp_worker::prelude::*;
use proto::backend::{self, cluster::PoolType, pkg::*};
use rand::Rng;

struct ProvisionResponse {
	provider_server_id: String,
	public_ip: String,
}

#[worker(name = "cluster-server-provision")]
async fn worker(
	ctx: &OperationContext<cluster::msg::server_provision::Message>,
) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;

	let datacenter_id = unwrap!(ctx.datacenter_id);
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(ctx.pool_type));
	let provider = unwrap!(backend::cluster::Provider::from_i32(ctx.provider));

	// Check if server is already provisioned
	// NOTE: sql record already exists before this worker is called
	let (provider_server_id, vlan_ip) = sql_fetch_one!(
		[ctx, (Option<String>, Option<String>), &crdb]
		"
		SELECT
			provider_server_id, vlan_ip
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;
	if let Some(provider_server_id) = provider_server_id {
		tracing::error!(
			?server_id,
			?provider_server_id,
			"server is already provisioned"
		);
		return Ok(());
	};

	// Fetch datacenter config
	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id],
	})
	.await?;
	let datacenter = unwrap!(datacenter_res.datacenters.first());
	let pool = unwrap!(
		datacenter
			.pools
			.iter()
			.find(|p| p.pool_type == ctx.pool_type),
		"datacenter does not have this type of pool configured"
	);

	// If the vlan_ip is set in the database but this message is being run again, that means an attempt to
	// provision servers failed. Use the vlan_ip that was determined before, otherwise find a new vlan_ip
	// to use
	let vlan_ip = if let Some(vlan_ip) = vlan_ip {
		vlan_ip
	} else {
		get_vlan_ip(ctx, &crdb, server_id, pool_type).await?
	};

	let provision_res = match provider {
		backend::cluster::Provider::Linode => {
			let mut hardware_list = pool.hardware.iter();

			// Iterate through list of hardware and attempt to schedule a server. Goes to the next
			// hardware if an error happens during provisioning
			loop {
				// List exhausted
				let Some(hardware) = hardware_list.next() else {
					break None;
				};

				tracing::info!(
					"attempting to provision hardware: {}",
					hardware.provider_hardware
				);

				let res = op!([ctx] linode_server_provision {
					server_id: ctx.server_id,
					provider_datacenter_id: datacenter.provider_datacenter_id.clone(),
					hardware: Some(hardware.clone()),
					pool_type: ctx.pool_type,
					vlan_ip: vlan_ip.clone(),
				})
				.await;

				match res {
					Ok(res) => {
						break Some(ProvisionResponse {
							provider_server_id: res.provider_server_id.clone(),
							public_ip: res.public_ip.clone(),
						})
					}
					Err(err) => {
						tracing::warn!(?err, "failed to provision server, cleaning up");

						destroy_server(ctx, server_id).await?;
					}
				}
			}
		}
	};

	// Update DB regardless of success (have to set vlan_ip)
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
		provision_res.as_ref().map(|res| &res.provider_server_id),
		vlan_ip,
		provision_res.as_ref().map(|res| &res.public_ip),
	)
	.await?;

	if provision_res.is_none() {
		tracing::info!(?server_id, "failed to provision server");
		bail!("failed to provision server");
	}

	// Install components
	msg!([ctx] cluster::msg::server_install(server_id) {
		server_id: ctx.server_id,
	})
	.await?;

	// Create DNS record
	if matches!(pool_type, backend::cluster::PoolType::Gg) {
		msg!([ctx] cluster::msg::server_dns_create(server_id) {
			server_id: ctx.server_id,
		})
		.await?;
	}

	Ok(())
}

async fn get_vlan_ip(
	ctx: &OperationContext<cluster::msg::server_provision::Message>,
	crdb: &CrdbPool,
	server_id: Uuid,
	pool_type: backend::cluster::PoolType,
) -> GlobalResult<String> {
	// Find next available vlan index
	let mut vlan_addr_range = match pool_type {
		PoolType::Job => util::net::job::vlan_addr_range(),
		PoolType::Gg => util::net::gg::vlan_addr_range(),
		PoolType::Ats => util::net::ats::vlan_addr_range(),
	};
	let max_idx = vlan_addr_range.count() as i64;
	let (network_idx,) = sql_fetch_one!(
		[ctx, (i64,), &crdb]
		"
		WITH
			get_next_network_idx AS (
				SELECT idx
				FROM generate_series(0, $2) AS s(idx)
				WHERE NOT EXISTS (
					SELECT 1
					FROM db_cluster.servers
					WHERE
						pool_type = $3 AND
						network_idx = mod(idx + $1, $2)
				)
				LIMIT 1
			),
			update_network_idx AS (
				UPDATE db_cluster.servers
				SET network_idx = mod((SELECT idx FROM get_next_network_idx) + $1, $2)
				WHERE server_id = $4
				RETURNING 1
			)
		SELECT idx FROM get_next_network_idx
		",
		// Choose a random index to start from for better index spread
		rand::thread_rng().gen_range(0i64..max_idx),
		max_idx,
		pool_type as i64,
		server_id
	)
	.await?;

	let vlan_ip = unwrap!(vlan_addr_range.nth(network_idx as usize));

	Ok(vlan_ip.to_string())
}

async fn destroy_server(
	ctx: &OperationContext<cluster::msg::server_provision::Message>,
	server_id: Uuid,
) -> GlobalResult<()> {
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers
		SET cloud_destroy_ts = $2
		WHERE
			server_id = $1
		",
		&server_id,
		util::timestamp::now(),
	)
	.await?;

	msg!([ctx] cluster::msg::server_destroy(server_id) {
		server_id: Some(server_id.into()),
	})
	.await?;

	Ok(())
}
