use anyhow::*;
use rivet_test_deps_docker::{TestDatabase, TestPubSub};
use std::{sync::Arc, time::Duration};
use universalpubsub::{NextOutput, PubSub, PublishOpts};
use uuid::Uuid;

fn setup_logging() {
	let _ = tracing_subscriber::fmt()
		.with_env_filter("debug")
		.with_ansi(false)
		.with_test_writer()
		.try_init();
}

#[tokio::test]
async fn test_nats_driver_with_memory_reconnect() {
	setup_logging();

	let test_id = Uuid::new_v4();
	let (pubsub_config, docker_config) = TestPubSub::Nats.config(test_id, 1).await.unwrap();
	let mut docker = docker_config.unwrap();
	docker.start().await.unwrap();
	tokio::time::sleep(Duration::from_secs(1)).await;

	let rivet_config::config::PubSub::Nats(nats) = pubsub_config else {
		unreachable!();
	};

	use std::str::FromStr;
	let server_addrs = nats
		.addresses
		.iter()
		.map(|addr| format!("nats://{addr}"))
		.map(|url| async_nats::ServerAddr::from_str(url.as_ref()))
		.collect::<Result<Vec<_>, _>>()
		.unwrap();

	let driver = universalpubsub::driver::nats::NatsDriver::connect(
		async_nats::ConnectOptions::new(),
		&server_addrs[..],
	)
	.await
	.unwrap();
	let pubsub = PubSub::new_with_memory_optimization(Arc::new(driver), true);

	test_reconnect_inner(&pubsub, &docker).await;
}

#[tokio::test]
async fn test_nats_driver_without_memory_reconnect() {
	setup_logging();

	let test_id = Uuid::new_v4();
	let (pubsub_config, docker_config) = TestPubSub::Nats.config(test_id, 1).await.unwrap();
	let mut docker = docker_config.unwrap();
	docker.start().await.unwrap();
	tokio::time::sleep(Duration::from_secs(1)).await;

	let rivet_config::config::PubSub::Nats(nats) = pubsub_config else {
		unreachable!();
	};

	use std::str::FromStr;
	let server_addrs = nats
		.addresses
		.iter()
		.map(|addr| format!("nats://{addr}"))
		.map(|url| async_nats::ServerAddr::from_str(url.as_ref()))
		.collect::<Result<Vec<_>, _>>()
		.unwrap();

	let driver = universalpubsub::driver::nats::NatsDriver::connect(
		async_nats::ConnectOptions::new(),
		&server_addrs[..],
	)
	.await
	.unwrap();
	let pubsub = PubSub::new_with_memory_optimization(Arc::new(driver), false);

	test_reconnect_inner(&pubsub, &docker).await;
}

#[tokio::test]
async fn test_postgres_driver_with_memory_reconnect() {
	setup_logging();

	let test_id = Uuid::new_v4();
	let (db_config, docker_config) = TestDatabase::Postgres.config(test_id, 1).await.unwrap();
	let mut docker = docker_config.unwrap();
	docker.start().await.unwrap();
	tokio::time::sleep(Duration::from_secs(5)).await;

	let rivet_config::config::Database::Postgres(pg) = db_config else {
		unreachable!();
	};
	let url = pg.url.read().clone();

	let driver = universalpubsub::driver::postgres::PostgresDriver::connect(url, true)
		.await
		.unwrap();
	let pubsub = PubSub::new_with_memory_optimization(Arc::new(driver), true);

	test_reconnect_inner(&pubsub, &docker).await;
}

#[tokio::test]
async fn test_postgres_driver_without_memory_reconnect() {
	setup_logging();

	let test_id = Uuid::new_v4();
	let (db_config, docker_config) = TestDatabase::Postgres.config(test_id, 1).await.unwrap();
	let mut docker = docker_config.unwrap();
	docker.start().await.unwrap();
	tokio::time::sleep(Duration::from_secs(5)).await;

	let rivet_config::config::Database::Postgres(pg) = db_config else {
		unreachable!();
	};
	let url = pg.url.read().clone();

	let driver = universalpubsub::driver::postgres::PostgresDriver::connect(url, false)
		.await
		.unwrap();
	let pubsub = PubSub::new_with_memory_optimization(Arc::new(driver), false);

	test_reconnect_inner(&pubsub, &docker).await;
}

async fn test_reconnect_inner(pubsub: &PubSub, docker: &rivet_test_deps_docker::DockerRunConfig) {
	tracing::info!("testing reconnect functionality");

	// Open subscription
	let mut subscriber = pubsub.subscribe("test.reconnect").await.unwrap();
	tracing::info!("opened initial subscription");

	// Test publish/receive message before restart
	let message_before = b"message before restart";
	pubsub
		.publish("test.reconnect", message_before, PublishOpts::broadcast())
		.await
		.unwrap();
	pubsub.flush().await.unwrap();

	match subscriber.next().await.unwrap() {
		NextOutput::Message(msg) => {
			assert_eq!(
				msg.payload, message_before,
				"message before restart should match"
			);
			tracing::info!("received message before restart");
		}
		NextOutput::Unsubscribed => {
			panic!("unexpected unsubscribe before restart");
		}
	}

	// Restart container
	tracing::info!("restarting docker container");
	docker.restart().await.unwrap();

	// Give the service time to come back up
	tokio::time::sleep(Duration::from_secs(3)).await;
	tracing::info!("docker container restarted");

	// Test publish/receive message after restart
	let message_after = b"message after restart";

	// Retry logic for publish after restart since connection might need to reconnect
	let mut retries = 0;
	const MAX_RETRIES: u32 = 10;
	loop {
		match pubsub
			.publish("test.reconnect", message_after, PublishOpts::broadcast())
			.await
		{
			Result::Ok(_) => {
				tracing::info!("published message after restart");
				break;
			}
			Result::Err(e) if retries < MAX_RETRIES => {
				retries += 1;
				tracing::debug!(?e, retries, "failed to publish after restart, retrying");
				tokio::time::sleep(Duration::from_millis(500)).await;
			}
			Result::Err(e) => {
				panic!("failed to publish after {} retries: {}", MAX_RETRIES, e);
			}
		}
	}

	pubsub.flush().await.unwrap();

	// Try to receive with timeout to handle reconnection delays
	let receive_timeout = Duration::from_secs(10);
	let receive_result = tokio::time::timeout(receive_timeout, subscriber.next()).await;

	match receive_result {
		Result::Ok(Result::Ok(NextOutput::Message(msg))) => {
			assert_eq!(
				msg.payload, message_after,
				"message after restart should match"
			);
			tracing::info!("received message after restart - reconnection successful");
		}
		Result::Ok(Result::Ok(NextOutput::Unsubscribed)) => {
			panic!("unexpected unsubscribe after restart");
		}
		Result::Ok(Result::Err(e)) => {
			panic!("error receiving message after restart: {}", e);
		}
		Result::Err(_) => {
			panic!("timeout receiving message after restart");
		}
	}

	tracing::info!("reconnect test completed successfully");
}
