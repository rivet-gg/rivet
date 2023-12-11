use std::net::Ipv4Addr;

use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker_test]
async fn server_install(ctx: TestCtx) {
	let server_id = Uuid::new_v4();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();
	let pool_type = backend::cluster::PoolType::Job;

	let (dc, vlan_ip) = setup(&ctx, server_id, datacenter_id, cluster_id, pool_type).await;

	// Create server
	let res = op!([ctx] linode_server_provision {
		server_id: Some(server_id.into()),
		provider_datacenter_id: dc.provider_datacenter_id.clone(),
		hardware: dc.hardware.first().cloned(),
		pool_type: pool_type as i32,
		vlan_ip: vlan_ip.to_string(),
	})
	.await
	.unwrap();

	// Set as provisioned
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers
		SET
			provider_server_id = $1,
			public_ip = $2
		WHERE server_id = $3
		",
		&res.provider_server_id,
		&res.public_ip,
		server_id,
	)
	.await
	.unwrap();

	msg!([ctx] cluster::msg::server_install(server_id) -> cluster::msg::server_install_complete {
		server_id: Some(server_id.into()),
	})
	.await
	.unwrap();

	// Destroy server after test is complete so we don't litter
	op!([ctx] linode_server_destroy {
		server_id: Some(server_id.into()),
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
) -> (backend::cluster::Datacenter, Ipv4Addr) {
	let dc = backend::cluster::Datacenter {
		datacenter_id: Some(datacenter_id.into()),
		cluster_id: Some(cluster_id.into()),
		name_id: util::faker::ident(),
		display_name: util::faker::ident(),
		coords: Some(backend::net::Coordinates {
			latitude: 0.0,
			longitude: 0.0,
		}),
		hardware: vec![backend::cluster::Hardware {
			provider_hardware: "g6-nanode-1".to_string(),
		}],

		provider: backend::cluster::Provider::Linode as i32,
		provider_datacenter_id: "us-southeast".to_string(),

		pools: Vec::new(),
		drain_timeout: 0,
	};

	// TODO: This might collide if the test fails, its static
	let vlan_ip = util::net::job::vlan_addr_range().last().unwrap();

	msg!([ctx] cluster::msg::create(cluster_id) -> cluster::msg::create_complete {
		cluster_id: Some(cluster_id.into()),
		name_id: util::faker::ident(),
		owner_team_id: None,
	})
	.await
	.unwrap();

	// Insert fake record to appease foreign key constraint (both sql calls in this test are normally done
	// by `cluster-server-provision`)
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.datacenters (
			datacenter_id,
			cluster_id,
			config
		)
		VALUES ($1, $2, $3)
		",
		datacenter_id,
		cluster_id,
		{
			let mut buf = Vec::with_capacity(dc.encoded_len());
			dc.encode(&mut buf)?;

			buf
		},
	)
	.await
	.unwrap();
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.servers (
			server_id,
			datacenter_id,
			cluster_id,
			pool_type,
			create_ts,
			vlan_ip
		)
		VALUES ($1, $2, $3, $4, $5, $6)
		",
		server_id,
		datacenter_id,
		cluster_id,
		pool_type as i32 as i64,
		util::timestamp::now(),
		vlan_ip.to_string(),
	)
	.await
	.unwrap();

	(dc, vlan_ip)
}
