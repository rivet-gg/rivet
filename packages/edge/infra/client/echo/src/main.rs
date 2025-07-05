use std::{env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::*;
use futures_util::{SinkExt, StreamExt};
use pegboard_runner_protocol as rp;
use tokio::{net::UnixStream, sync::Mutex};
use tokio_util::codec::Framed;
use warp::Filter;

const PING_INTERVAL: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() {
	// Print all environment variables
	println!("Environment variables:");
	for (key, value) in env::vars() {
		println!("  {}: {}", key, value);
	}

	// Get manager connection details from env vars
	let manager_socket_path = PathBuf::from(
		env::var("RIVET_MANAGER_SOCKET_PATH").expect("RIVET_MANAGER_SOCKET_PATH not set"),
	);

	// Get HTTP server port from env var or use default
	let http_port = env::var("PORT_MAIN")
		.expect("PORT_MAIN not set")
		.parse::<u16>()
		.expect("bad PORT_MAIN");

	// Spawn the unix socket client
	tokio::spawn(async move {
		if let Err(e) = run_socket_client(manager_socket_path).await {
			eprintln!("Socket client error: {}", e);
		}
	});

	// Start the HTTP server
	let http_addr: SocketAddr = ([0, 0, 0, 0], http_port).into();
	println!("Starting HTTP server on {}", http_addr);

	// Define the echo route
	let echo = warp::any().and(warp::body::bytes()).map(|body| {
		println!("Received HTTP request");

		http::response::Builder::new()
			.status(warp::http::StatusCode::OK)
			.body(body)
			.unwrap()
	});

	// Start the server
	warp::serve(echo).run(http_addr).await;
}

async fn run_socket_client(socket_path: PathBuf) -> Result<()> {
	println!("Connecting to socket at {}", socket_path.display());

	// Connect to the socket server
	let stream = UnixStream::connect(socket_path).await?;
	println!("Socket connection established");

	let framed = Framed::new(stream, rp::codec());

	// Split the stream
	let (write, mut read) = framed.split();

	// Ping thread
	let write = Arc::new(Mutex::new(write));
	let write2 = write.clone();
	tokio::spawn(async move {
		loop {
			tokio::time::sleep(PING_INTERVAL).await;

			let payload = rp::proto::ToManager {
				message: Some(rp::proto::to_manager::Message::Ping(
					rp::proto::to_manager::Ping {},
				)),
			};

			if write2
				.lock()
				.await
				.send(rp::encode_frame(&payload).unwrap().into())
				.await
				.is_err()
			{
				break;
			}
		}
	});

	// Process incoming messages
	while let Some(frame) = read.next().await.transpose()? {
		let (_, packet) = rp::decode_frame::<rp::proto::ToRunner>(&frame.freeze())
			.context("failed to decode frame")?;
		println!("Received packet: {packet:?}");

		match packet.message.context("ToRunner.message")? {
			rp::proto::to_runner::Message::Init(_) => {}
			rp::proto::to_runner::Message::Pong(_) => {}
			rp::proto::to_runner::Message::Close(msg) => {
				println!("Received close, stopping: {:?}", msg.reason);
				break;
			}
			rp::proto::to_runner::Message::StartActor(msg) => {
				let payload = rp::proto::ToManager {
					message: Some(rp::proto::to_manager::Message::ActorStateUpdate(
						rp::proto::to_manager::ActorStateUpdate {
							actor_id: msg.actor_id.clone(),
							generation: msg.generation,
							state: Some(rp::proto::ActorState {
								state: Some(rp::proto::actor_state::State::Running(
									rp::proto::actor_state::Running {},
								)),
							}),
						},
					)),
				};

				write
					.lock()
					.await
					.send(
						rp::encode_frame(&payload)
							.context("failed to encode frame")?
							.into(),
					)
					.await?;

				// KV get request
				let payload = rp::proto::ToManager {
					message: Some(rp::proto::to_manager::Message::Kv(
						rp::proto::kv::Request {
							actor_id: msg.actor_id,
							generation: msg.generation,
							request_id: 0,
							data: Some(rp::proto::kv::request::Data::Get(
								rp::proto::kv::request::Get {
									keys: vec![rp::proto::kv::Key {
										segments: vec![vec![0, 1, 2], vec![3, 4, 5]],
									}],
								},
							)),
						},
					)),
				};

				write
					.lock()
					.await
					.send(
						rp::encode_frame(&payload)
							.context("failed to encode frame")?
							.into(),
					)
					.await?;
			}
			rp::proto::to_runner::Message::SignalActor(msg) => {
				let payload = rp::proto::ToManager {
					message: Some(rp::proto::to_manager::Message::ActorStateUpdate(
						rp::proto::to_manager::ActorStateUpdate {
							actor_id: msg.actor_id,
							generation: msg.generation,
							state: Some(rp::proto::ActorState {
								state: Some(rp::proto::actor_state::State::Exited(
									rp::proto::actor_state::Exited { exit_code: None },
								)),
							}),
						},
					)),
				};

				write
					.lock()
					.await
					.send(rp::encode_frame(&payload)?.into())
					.await?;
			}
			rp::proto::to_runner::Message::Kv(_msg) => {
				// TODO:
			}
		}
	}

	println!("Socket connection closed");
	Ok(())
}
