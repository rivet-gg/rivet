use chirp_worker::prelude::*;
use proto::backend;

#[worker_test]
async fn basic(ctx: TestCtx) {
	let server_id = Uuid::new_v4();
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();
	let pool_type = backend::cluster::PoolType::Job as i32;

	// Insert fake records
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.clusters (
			cluster_id,
			config,
			create_ts
		)
		VALUES ($1, $2, $3)
		",
		cluster_id,
		Vec::<u8>::new(),
		util::timestamp::now(),
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
	})
	.await
	.unwrap();

	// Set as provisioned
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers
		SET provider_server_id = $1
		WHERE server_id = $2
		",
		&res.provider_server_id,
		server_id,
	)
	.await
	.unwrap();

	op!([ctx] linode_server_destroy {
		server_id: Some(server_id.into()),
	})
	.await
	.unwrap();

	// Should do nothing
	op!([ctx] linode_server_destroy {
		server_id: Some(server_id.into()),
	})
	.await
	.unwrap();
}
