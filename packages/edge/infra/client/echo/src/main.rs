use std::{env, net::SocketAddr, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;
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
	let manager_ip = env::var("RIVET_MANAGER_IP").expect("RIVET_MANAGER_IP not set");
	let manager_port = env::var("RIVET_MANAGER_PORT").expect("RIVET_MANAGER_PORT not set");
	let manager_addr = format!("ws://{}:{}", manager_ip, manager_port);

	// Get HTTP server port from env var or use default
	let http_port = env::var("PORT_MAIN")
		.expect("PORT_MAIN not set")
		.parse::<u16>()
		.expect("bad PORT_MAIN");

	// Spawn the WebSocket client
	tokio::spawn(async move {
		if let Err(e) = run_websocket_client(&manager_addr).await {
			eprintln!("WebSocket client error: {}", e);
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

async fn run_websocket_client(url: &str) -> Result<(), Box<dyn std::error::Error>> {
	println!("Connecting to WebSocket at {}", url);

	// Connect to the WebSocket server
	let (ws_stream, _) = connect_async(url).await?;
	println!("WebSocket connection established");

	// Split the stream
	let (mut write, mut read) = ws_stream.split();

	let payload = json!({
		"init": {
			"runner_id": Uuid::nil(),
		},
	});

	let data = serde_json::to_vec(&payload)?;
	write.send(Message::Binary(data)).await?;
	println!("Sent init message");

	// Ping thread
	let write = Arc::new(Mutex::new(write));
	let write2 = write.clone();
	tokio::spawn(async move {
		loop {
			tokio::time::sleep(PING_INTERVAL).await;

			if write2
				.lock()
				.await
				.send(Message::Ping(Vec::new()))
				.await
				.is_err()
			{
				break;
			}
		}
	});

	// Process incoming messages
	while let Some(message) = read.next().await {
		match message {
			Ok(msg) => match msg {
				Message::Pong(_) => {}
				Message::Binary(buf) => {
					let packet = serde_json::from_slice::<serde_json::Value>(&buf)?;
					println!("Received packet: {packet:?}");

					if let Some(packet) = packet.get("start_actor") {
						let payload = json!({
							"actor_state_update": {
								"actor_id": packet["actor_id"],
								"generation": packet["generation"],
								"state": {
									"running": null,
								},
							},
						});

						let data = serde_json::to_vec(&payload)?;
						write.lock().await.send(Message::Binary(data)).await?;
					} else if let Some(packet) = packet.get("signal_actor") {
						let payload = json!({
							"actor_state_update": {
								"actor_id": packet["actor_id"],
								"generation": packet["generation"],
								"state": {
									"exited": {
										"exit_code": null,
									},
								},
							},
						});

						let data = serde_json::to_vec(&payload)?;
						write.lock().await.send(Message::Binary(data)).await?;
					}
				}
				msg => eprintln!("Unexpected message: {msg:?}"),
			},
			Err(e) => {
				eprintln!("Error reading message: {}", e);
				break;
			}
		}
	}

	println!("WebSocket connection closed");
	Ok(())
}
