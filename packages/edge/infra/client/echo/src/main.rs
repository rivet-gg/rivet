use std::{
	env,
	io::{Cursor, Write},
	net::SocketAddr,
	path::PathBuf,
	sync::Arc,
	time::Duration,
};

use anyhow::*;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tokio::{net::UnixStream, sync::Mutex};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
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

	let codec = LengthDelimitedCodec::builder()
		.length_field_type::<u32>()
		.length_field_length(4)
		// No offset
		.length_field_offset(0)
		// Header length is not included in the length calculation
		.length_adjustment(4)
		// Skip length, but header is included in the returned bytes
		.num_skip(4)
		.new_codec();

	let framed = Framed::new(stream, codec);

	// Split the stream
	let (write, mut read) = framed.split();

	// Ping thread
	let write = Arc::new(Mutex::new(write));
	let write2 = write.clone();
	tokio::spawn(async move {
		loop {
			tokio::time::sleep(PING_INTERVAL).await;

			let payload = json!({
				"ping": null
			});

			if write2
				.lock()
				.await
				.send(encode_frame(&payload).unwrap())
				.await
				.is_err()
			{
				break;
			}
		}
	});

	// Process incoming messages
	while let Some(frame) = read.next().await.transpose()? {
		let (_, packet) =
			decode_frame::<serde_json::Value>(&frame.freeze()).context("failed to decode frame")?;
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

			write
				.lock()
				.await
				.send(encode_frame(&payload).context("failed to encode frame")?)
				.await?;
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

			write.lock().await.send(encode_frame(&payload)?).await?;
		}
	}

	println!("Socket connection closed");
	Ok(())
}

fn encode_frame<T: Serialize>(payload: &T) -> Result<Bytes> {
	let mut buf = Vec::with_capacity(4);
	let mut cursor = Cursor::new(&mut buf);

	cursor.write(&[0u8; 4])?; // header (currently unused)

	serde_json::to_writer(&mut cursor, payload)?;

	cursor.flush()?;

	Ok(buf.into())
}

fn decode_frame<T: DeserializeOwned>(frame: &Bytes) -> Result<([u8; 4], T)> {
	ensure!(frame.len() >= 4, "Frame too short");

	// Extract the header (first 4 bytes)
	let header = [frame[0], frame[1], frame[2], frame[3]];

	// Deserialize the rest of the frame (payload after the header)
	let payload = serde_json::from_slice(&frame[4..])?;

	Ok((header, payload))
}
