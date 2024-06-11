use std::collections::HashMap;

use chirp_worker::prelude::*;
use proto::backend;

#[worker_test]
async fn create(ctx: TestCtx) {
	let game_res = op!([ctx] faker_game {
		..Default::default()
	})
	.await
	.unwrap();
	let game_id = game_res.game_id.unwrap();

	// Pick an existing cluster
	let cluster_id = op!([ctx] cluster_list {})
		.await
		.unwrap()
		.cluster_ids
		.first()
		.unwrap()
		.to_owned();

	// Pick an existing datacenter
	let datacenter_id = op!([ctx] cluster_datacenter_list {
		cluster_ids: vec![cluster_id],
	})
	.await
	.unwrap()
	.clusters
	.first()
	.unwrap()
	.datacenter_ids
	.first()
	.unwrap()
	.to_owned();

	let build_res: backend::pkg::faker::build::Response = op!([ctx] faker_build {
		game_id: Some(game_id),
		image: backend::faker::Image::DsEcho as i32,
	})
	.await
	.unwrap();

	let runtime = Some(
		proto::backend::pkg::dynamic_servers::server_create::request::Runtime::DockerRuntime(
			proto::backend::dynamic_servers::DockerRuntime {
				args: Vec::new(),
				environment: vec![
					("some_envkey_test".to_string(), "2134523".to_string()),
					(
						"some_other_envkey_test".to_string(),
						"4325234356".to_string(),
					),
				]
				.into_iter()
				.collect(),
				image_id: Some(build_res.build_id.unwrap()),
				network: Some(proto::backend::dynamic_servers::DockerNetwork {
					mode: 0,
					ports: vec![(
						"testing2".to_string(),
						backend::dynamic_servers::DockerPort {
							port: Some(28234),
							routing: Some(
								backend::dynamic_servers::docker_port::Routing::GameGuard(
									backend::dynamic_servers::DockerGameGuardRouting {
										protocol: 0,
									},
								),
							),
						},
					)]
					// Collect into hashmap
					.into_iter()
					.collect(),
				}),
			},
		),
	);

	let faker_region = op!([ctx] faker_region {}).await.unwrap();

	let server = op!([ctx] ds_server_create {
		game_id: Some(game_id),
		cluster_id: Some(cluster_id),
		datacenter_id: faker_region.region_id,
		resources: Some(proto::backend::dynamic_servers::ServerResources { cpu_millicores: 100, memory_mib: 200 }),
		kill_timeout_ms: 0,
		metadata: HashMap::new(),
		runtime: runtime,
	})
	.await
	.unwrap()
	.server
	.unwrap();

	// TODO: Switch this
	// let hostname = format!(
	// 	"{}-{}.server.{}.rivet.run",
	// 	server.server_id.unwrap(),
	// 	"1234",
	// 	faker_region.region_id.unwrap()
	// );

	let hostname = format!(
		"{}-{}.lobby.{}.{}",
		server.server_id.unwrap(),
		"testing2",
		faker_region.region_id.unwrap(),
		util::env::domain_job().unwrap(),
	);

	// Async sleep for 5 seconds
	tokio::time::sleep(std::time::Duration::from_secs(30)).await;

	tracing::info!(?hostname, "hostest mostest");

	// Echo body
	let random_body = Uuid::new_v4().to_string();
	let client = reqwest::Client::new();
	let res = client
		.post(format!("http://{hostname}"))
		.body(random_body.clone())
		.send()
		.await
		.unwrap()
		.error_for_status()
		.unwrap();
	let res_text = res.text().await.unwrap();
	assert_eq!(random_body, res_text, "echoed wrong response");

	// assert_eq!(game_res.game_id.unwrap(), server.game_id.unwrap().as_uuid());
}
