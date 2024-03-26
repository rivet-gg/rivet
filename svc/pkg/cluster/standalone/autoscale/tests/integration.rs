use std::collections::HashMap;

use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

use ::cluster_autoscale::run_from_env;

#[tokio::test(flavor = "multi_thread")]
async fn basic() {
	tracing_subscriber::fmt()
		.json()
		.with_max_level(tracing::Level::INFO)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
		.init();

	let ctx = TestCtx::from_env("cluster-autoscale-test").await.unwrap();

	let provision_margin = util::env::var("JOB_SERVER_PROVISION_MARGIN")
		.unwrap()
		.parse::<u32>()
		.unwrap();
	let setup = setup_cluster(&ctx).await;

	job_autoscale(&ctx, &setup, provision_margin).await;
	gg_autoscale(&ctx, &setup).await;
}

async fn job_autoscale(ctx: &TestCtx, setup_cluster: &SetupCluster, provision_margin: u32) {
	let pools = rivet_pools::from_env("cluster-autoscale-test")
		.await
		.unwrap();

	let job_run_res = op!([ctx] faker_job_run {
		region_id: Some(setup_cluster.datacenter_id.into()),
		..Default::default()
	})
	.await
	.unwrap();
	let run_id = job_run_res.run_id.unwrap().as_uuid();

	// Update the max count before the autoscaler is triggered to ensure the correct state
	msg!([ctx] cluster::msg::datacenter_update(setup_cluster.datacenter_id) -> cluster::msg::datacenter_scale {
		datacenter_id: Some(setup_cluster.datacenter_id.into()),
		pools: vec![cluster::msg::datacenter_update::PoolUpdate {
			pool_type: backend::cluster::PoolType::Job as i32,
			hardware: Vec::new(),
			desired_count: Some(1),
			max_count: Some(provision_margin + 4),
		}],
		drain_timeout: None,
	})
	.await
	.unwrap();

	run_from_env(pools.clone()).await.unwrap();
	tokio::time::sleep(std::time::Duration::from_secs(3)).await;

	let datacenters_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![setup_cluster.datacenter_id.into()],
	})
	.await
	.unwrap();
	let datacenter = datacenters_res.datacenters.first().unwrap();
	let job_pool = datacenter.pools.first().unwrap();

	assert_eq!(
		provision_margin + 1,
		job_pool.desired_count,
		"desired count did not update"
	);

	// Stop job
	msg!([ctx] job_run::msg::stop(run_id) -> job_run::msg::cleanup {
		run_id: Some(run_id.into()),
		..Default::default()
	})
	.await
	.unwrap();

	// Wait for job to fully stop
	tokio::time::sleep(std::time::Duration::from_secs(15)).await;

	run_from_env(pools).await.unwrap();
	tokio::time::sleep(std::time::Duration::from_secs(3)).await;

	let datacenters_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![setup_cluster.datacenter_id.into()],
	})
	.await
	.unwrap();
	let datacenter = datacenters_res.datacenters.first().unwrap();
	let job_pool = datacenter.pools.first().unwrap();

	assert_eq!(
		provision_margin, job_pool.desired_count,
		"desired count did not update"
	);

	// Clean up afterwards so we don't litter
	msg!([ctx] cluster::msg::datacenter_update(setup_cluster.datacenter_id) -> cluster::msg::datacenter_scale {
		datacenter_id: Some(setup_cluster.datacenter_id.into()),
		pools: vec![
			cluster::msg::datacenter_update::PoolUpdate {
				pool_type: backend::cluster::PoolType::Job as i32,
				hardware: Vec::new(),
				desired_count: Some(0),
				max_count: Some(0),
			},
			cluster::msg::datacenter_update::PoolUpdate {
				pool_type: backend::cluster::PoolType::Ats as i32,
				hardware: Vec::new(),
				desired_count: Some(0),
				max_count: Some(0),
			},
		],
		drain_timeout: None,
	})
	.await
	.unwrap();
}

async fn gg_autoscale(ctx: &TestCtx, setup_cluster: &SetupCluster) {
	// TODO: Figure out a way to spoof prometheus metrics
}

struct SetupCluster {
	datacenter_id: Uuid,
	job_server_id: Uuid,
	ats_server_id: Uuid,
}

async fn setup_cluster(ctx: &TestCtx) -> SetupCluster {
	let job_server_id = Uuid::new_v4();
	let ats_server_id = Uuid::new_v4();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();

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

		pools: vec![
			backend::cluster::Pool {
				pool_type: backend::cluster::PoolType::Job as i32,
				hardware: vec![backend::cluster::Hardware {
					provider_hardware: util_cluster::test::HARDWARE.to_string(),
				}],
				desired_count: 0,
				max_count: 0,
			},
			// Required for autoscaler to run
			backend::cluster::Pool {
				pool_type: backend::cluster::PoolType::Gg as i32,
				hardware: Vec::new(),
				desired_count: 0,
				max_count: 0,
			},
			backend::cluster::Pool {
				pool_type: backend::cluster::PoolType::Ats as i32,
				hardware: vec![backend::cluster::Hardware {
					provider_hardware: util_cluster::test::HARDWARE.to_string(),
				}],
				desired_count: 0,
				max_count: 0,
			},
		],

		build_delivery_method: backend::cluster::BuildDeliveryMethod::TrafficServer as i32,
		drain_timeout: 60,
	};

	msg!([ctx] cluster::msg::datacenter_create(datacenter_id) -> cluster::msg::datacenter_scale {
		config: Some(dc.clone()),
	})
	.await
	.unwrap();

	// Write new servers to db
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.servers (
			server_id,
			datacenter_id,
			cluster_id,
			pool_type,
			create_ts
		)
		VALUES
			($1, $3, $4, $5, $7),
			($2, $3, $4, $6, $7)
		",
		job_server_id,
		ats_server_id,
		datacenter_id,
		cluster_id,
		backend::cluster::PoolType::Job as i32,
		backend::cluster::PoolType::Ats as i32,
		util::timestamp::now(),
	)
	.await
	.unwrap();

	// We update the datacenter AFTER inserting the rows so that nothing is provisioned by datacenter_scale.
	// This is because we manually create the servers here so they have a "test" tag
	msg!([ctx] cluster::msg::datacenter_update(datacenter_id) -> cluster::msg::datacenter_scale {
		datacenter_id: Some(datacenter_id.into()),
		pools: vec![
			cluster::msg::datacenter_update::PoolUpdate {
				pool_type: backend::cluster::PoolType::Job as i32,
				hardware: Vec::new(),
				desired_count: Some(1),
				max_count: Some(1),
			},
			cluster::msg::datacenter_update::PoolUpdate {
				pool_type: backend::cluster::PoolType::Ats as i32,
				hardware: Vec::new(),
				desired_count: Some(1),
				max_count: Some(1),
			},
		],
		drain_timeout: None,
	})
	.await
	.unwrap();

	// Provision both servers
	msg!([ctx] cluster::msg::server_provision(ats_server_id) {
		cluster_id: Some(cluster_id.into()),
		datacenter_id: Some(datacenter_id.into()),
		server_id: Some(ats_server_id.into()),
		pool_type: backend::cluster::PoolType::Ats as i32,
		provider: dc.provider,
		tags: vec!["test".to_string()],
	})
	.await
	.unwrap();
	// Wait until node is registered (we assume the ats server provisioned above will be ready by then)
	msg!([ctx] @notrace cluster::msg::server_provision(job_server_id) -> nomad::msg::monitor_node_registered {
		cluster_id: Some(cluster_id.into()),
		datacenter_id: Some(datacenter_id.into()),
		server_id: Some(job_server_id.into()),
		pool_type: backend::cluster::PoolType::Job as i32,
		provider: dc.provider,
		tags: vec!["test".to_string()],
	})
	.await
	.unwrap();

	SetupCluster {
		datacenter_id,
		job_server_id,
		ats_server_id,
	}
}
