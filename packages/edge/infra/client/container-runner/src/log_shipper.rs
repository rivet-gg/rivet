use std::{io::Write, net::TcpStream, sync::mpsc, thread::JoinHandle};

use anyhow::*;
use serde::Serialize;
use serde_json;
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum StreamType {
	StdOut = 0,
	StdErr = 1,
}

pub struct ReceivedMessage {
	pub stream_type: StreamType,
	pub ts: u64,
	pub message: String,
}

/// Sends logs from the container to the Vector agent on the machine.
///
/// This will run until the `msg_rx` sender is dropped before shutting down.
///
/// If attempting to reconnect while the runner is shut down, this will exit immediately, dropping
/// all logs in the process. This is to ensure that if Vector becomes unreachable, we don't end up
/// with a lot of lingering runners that refuse to exit.
pub struct LogShipper {
	/// Notifies of process shutdown.
	pub shutdown_rx: mpsc::Receiver<()>,

	/// Receiver for messages to be shipped. This holds a buffer of messages waiting to be send.
	///
	/// If the socket closes or creates back pressure, logs will be dropped on the main thread when
	/// trying to send to this channel.
	pub msg_rx: mpsc::Receiver<ReceivedMessage>,

	pub vector_socket_addr: String,

	pub actor_id: String,

	pub env_id: Uuid,
}

impl LogShipper {
	pub fn spawn(self) -> JoinHandle<()> {
		std::thread::spawn(move || self.run())
	}

	fn run(self) {
		// Retry loop
		loop {
			match self.run_inner() {
				Result::Ok(()) => {
					println!("Exiting log shipper");
					break;
				}
				Err(err) => {
					eprintln!("Log shipper error: {err:?}");

					// Wait before attempting to reconnect. Wait for disconnect in this time
					// period.
					match self
						.shutdown_rx
						.recv_timeout(std::time::Duration::from_secs(15))
					{
						Result::Ok(_) => {
							println!("Log shipper received shutdown");
							break;
						}
						Err(mpsc::RecvTimeoutError::Disconnected) => {
							eprintln!("Log shipper shutdown unexpectedly disconnected");
							break;
						}
						Err(mpsc::RecvTimeoutError::Timeout) => {
							// Not shut down, attempt reconnect
						}
					}
				}
			}
		}
	}

	fn run_inner(&self) -> Result<()> {
		println!(
			"Connecting log shipper to Vector at {}",
			self.vector_socket_addr
		);

		let mut stream = TcpStream::connect(&self.vector_socket_addr)?;

		println!("Log shipper connected");

		while let Result::Ok(message) = self.msg_rx.recv() {
			let vector_message = VectorMessage::Actors {
				actor_id: self.actor_id.as_str(),
				env_id: self.env_id,
				stream_type: message.stream_type as u8,
				ts: message.ts,
				message: message.message.as_str(),
			};

			serde_json::to_writer(&mut stream, &vector_message)?;
			stream.write_all(b"\n")?;
		}

		println!("Log shipper msg_rx disconnected");

		Ok(())
	}
}

/// Vector-compatible message format
#[derive(Serialize)]
#[serde(tag = "source")]
enum VectorMessage<'a> {
	#[serde(rename = "actors")]
	Actors {
		actor_id: &'a str,
		env_id: Uuid,
		stream_type: u8,
		ts: u64,
		message: &'a str,
	},
}
