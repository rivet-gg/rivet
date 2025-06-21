import * as net from "net";
import * as fs from "fs";
import { setInterval, clearInterval } from "timers";

export function connectToManager() {
	const socketPath = process.env.RIVET_MANAGER_SOCKET_PATH;
	let pingInterval: NodeJS.Timeout;

	if (!socketPath) {
		console.error("Missing RIVET_MANAGER_SOCKET_PATH environment variable");
		return;
	}

	console.log(`Connecting to Unix socket at ${socketPath}`);

	// Ensure the socket path exists
	if (!fs.existsSync(socketPath)) {
		console.error(`Socket path does not exist: ${socketPath}`);
		return;
	}

	const client = net.createConnection(socketPath, () => {
		console.log("Socket connection established");

		// Start ping loop to keep connection alive
		pingInterval = setInterval(() => {
			const pingMessage = { ping: null };
			client.write(encodeFrame(pingMessage));
		}, 2000);
	});

	client.on("data", (data) => {
		const packets = decodeFrames(data);
		packets.forEach((packet) => {
			console.log("Received packet from manager:", packet);

			if (packet.start_actor) {
				const response = {
					actor_state_update: {
						actor_id: packet.start_actor.actor_id,
						generation: packet.start_actor.generation,
						state: {
							running: null,
						},
					},
				};
				client.write(encodeFrame(response));
			} else if (packet.signal_actor) {
				const response = {
					actor_state_update: {
						actor_id: packet.signal_actor.actor_id,
						generation: packet.signal_actor.generation,
						state: {
							exited: {
								exit_code: 0,
							},
						},
					},
				};
				client.write(encodeFrame(response));
			}
		});
	});

	client.on("error", (error) => {
		console.error("Socket error:", error);
	});

	client.on("close", () => {
		console.log("Socket connection closed, attempting to reconnect...");

		// Clear ping interval when connection closes
		if (pingInterval) clearInterval(pingInterval);

		setTimeout(connectToManager, 5000);
	});
}

function encodeFrame(payload: any): Buffer {
	const json = JSON.stringify(payload);
	const payloadLength = Buffer.alloc(4);
	payloadLength.writeUInt32BE(json.length, 0);

	const header = Buffer.alloc(4); // All zeros for now
	return Buffer.concat([payloadLength, header, Buffer.from(json)]);
}

function decodeFrames(buffer: Buffer): any[] {
	const packets = [];
	let offset = 0;

	while (offset < buffer.length) {
		if (buffer.length - offset < 8) break; // Incomplete frame length + header
		const payloadLength = buffer.readUInt32BE(offset);
		offset += 4;

		// Skip the header (4 bytes)
		offset += 4;

		if (buffer.length - offset < payloadLength) break; // Incomplete frame data
		const json = buffer.slice(offset, offset + payloadLength).toString();
		packets.push(JSON.parse(json));
		offset += payloadLength;
	}

	return packets;
}
