use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker_test]
async fn server_install(ctx: TestCtx) {
	if !util::feature::server_provision() {
		return;
	}

	let server_id = Uuid::new_v4();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();
	let pool_type = backend::cluster::PoolType::Ats;

	let mut installed_sub = subscribe!([ctx] cluster::msg::server_install_complete(server_id))
		.await
		.unwrap();

	setup(&ctx, server_id, datacenter_id, cluster_id, pool_type).await;

	// This can take several minutes to complete
	installed_sub.next().await.unwrap();

	// Clean up datacenter afterwards so we don't litter
	msg!([ctx] cluster::msg::datacenter_update(datacenter_id) {
		datacenter_id: Some(datacenter_id.into()),
		pools: vec![backend::cluster::Pool {
			pool_type: pool_type as i32,
			hardware: Vec::new(),
			desired_count: 0,
		}],
		..Default::default()
	})
	.await
	.unwrap();
}

async fn setup(
	ctx: &TestCtx,
	server_id: Uuid,
	datacenter_id: Uuid,
	cluster_id: Uuid,
	pool_type: backend::cluster::PoolType,
) -> backend::cluster::Datacenter {
	msg!([ctx] cluster::msg::create(cluster_id) -> cluster::msg::create_complete {
		cluster_id: Some(cluster_id.into()),
		name_id: util::faker::ident(),
		owner_team_id: None,
	})
	.await
	.unwrap();

	let dc = backend::cluster::Datacenter {
		datacenter_id: Some(datacenter_id.into()),
		cluster_id: Some(cluster_id.into()),
		name_id: util::faker::ident(),
		display_name: util::faker::ident(),

		provider: backend::cluster::Provider::Linode as i32,
		provider_datacenter_id: "us-southeast".to_string(),

		pools: vec![backend::cluster::Pool {
			pool_type: pool_type as i32,
			hardware: vec![backend::cluster::Hardware {
				provider_hardware: "g6-nanode-1".to_string(),
			}],
			desired_count: 1,
		}],

		build_delivery_method: backend::cluster::BuildDeliveryMethod::TrafficServer as i32,
		drain_timeout: 3600,
	};

	tracing::info!(?datacenter_id, "creating datacenter");
	msg!([ctx] cluster::msg::datacenter_create(datacenter_id) -> cluster::msg::datacenter_scale {
		config: Some(dc.clone()),
	})
	.await
	.unwrap();

	dc
}
