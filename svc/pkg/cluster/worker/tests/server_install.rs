use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker_test]
async fn server_install(ctx: TestCtx) {
	let server_id = Uuid::new_v4();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();
	let pool_type = backend::cluster::PoolType::Job as i32;

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
		INSERT INTO db_cluster.servers (
			server_id,
			datacenter_id,
			cluster_id,
			pool_type,
			create_ts
		)
		VALUES ($1, $2, $3, $4, $5)
		",
		server_id,
		datacenter_id,
		cluster_id,
		pool_type as i64,
		util::timestamp::now(),
	)
	.await
	.unwrap();

	// Create server
	let res = op!([ctx] linode_server_provision {
		server_id: Some(server_id.into()),
		provider_datacenter_id: "us-southeast".to_string(),
		hardware: Some(backend::cluster::Hardware {
			provider_hardware: "g6-nanode-1".to_string(),
		}),
		pool_type: pool_type,
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

	msg!([ctx] cluster::msg::server_install(cluster_id, datacenter_id, server_id) {
		server_id: Some(server_id.into()),
	})
	.await
	.unwrap();

	// tokio::time::sleep(std::time::Duration::from_secs(10)).await;

	// // Destroy server after test is complete so we don't litter
	// op!([ctx] linode_server_destroy {
	// 	server_id: Some(server_id.into()),
	// })
	// .await
	// .unwrap();
}

#[worker_test]
async fn test(ctx: TestCtx) {
	use std::str::FromStr;

	let server_id = Uuid::from_str("66348dbf-a9e7-4c14-a686-2a31444f2a3f").unwrap();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();

	msg!([ctx] cluster::msg::server_install(cluster_id, datacenter_id, server_id) {
		server_id: Some(server_id.into()),
	})
	.await
	.unwrap();
}
