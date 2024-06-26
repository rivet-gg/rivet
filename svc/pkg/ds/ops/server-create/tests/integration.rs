use std::collections::HashMap;

use chirp_worker::prelude::*;

#[worker_test]
async fn create(ctx: TestCtx) {
	let datacenter_id = Uuid::new_v4();
	let cluster_id = Uuid::new_v4();
	let game_id = Uuid::new_v4();
	let image_id = Uuid::new_v4();

	let server = op!([ctx] ds_server_create {
		game_id: Some(game_id.into()),
		cluster_id: Some(cluster_id.into()),
		datacenter_id: Some(datacenter_id.into()),
		resources: Some(proto::backend::dynamic_servers::ServerResources { cpu_millicores: 1000, memory_mib: 1000 }),
		kill_timeout_ms: 0,
		metadata: HashMap::new(),
		runtime: Some(
			proto::backend::pkg::dynamic_servers::server_create::request::Runtime::DockerRuntime(
				proto::backend::dynamic_servers::DockerRuntime {
					args: Vec::new(),
					environment: HashMap::new(),
					image_id: Some(image_id.into()),
					network: Some(
						proto::backend::dynamic_servers::DockerNetwork {
							mode: 1,
							ports: HashMap::new()
						}
					)
				}
			)
		),
	})
	.await
	.unwrap()
	.server
	.unwrap();

	assert_eq!(game_id, server.game_id.unwrap().as_uuid());
}
