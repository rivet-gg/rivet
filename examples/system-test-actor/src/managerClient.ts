import * as net from "net";
import * as fs from "fs";
import { setInterval, clearInterval } from "timers";
import * as util from "util";
import { encodeFrame, decodeFrames, types as protocol } from "@rivet-gg/runner-protocol";

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
			const pingMessage = new protocol.ToManager({
				ping: new protocol.ToManager.Ping()
			});
			client.write(encodeFrame(pingMessage));
		}, 2000);
	});

	client.on("data", (data) => {
		const packets = decodeFrames(data, protocol.ToRunner);

		for (let packet of packets) {
			console.log("Received packet from manager:", util.inspect(packet, { depth: null }));

			if (packet.start_actor) {
				const response = new protocol.ToManager({
					actor_state_update: new protocol.ToManager.ActorStateUpdate({
						actor_id: packet.start_actor.actor_id,
						generation: packet.start_actor.generation,
						state: new protocol.ActorState({
							running: new protocol.ActorState.Running()
						})
					})
				});
				client.write(encodeFrame(response));

				console.log(`actor_${packet.start_actor.actor_id}`, 'fweh');

				const kvMessage = new protocol.ToManager({
					kv: new rivet.pegboard.kv.Request({
						actor_id: packet.start_actor.actor_id,
						generation: packet.start_actor.generation,
						request_id: 1,
						put: new rivet.pegboard.kv.Request.Put({
							keys: [
								new rivet.pegboard.kv.Key({
									segments: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])]
								})
							],
							values: [
								new Uint8Array([11, 12, 13, 14, 15, 16])
							]
						})
					})
				});
				client.write(encodeFrame(kvMessage));

				const kvMessage2 = new protocol.ToManager({
					kv: new rivet.pegboard.kv.Request({
						actor_id: packet.start_actor.actor_id,
						generation: packet.start_actor.generation,
						request_id: 2,
						get: new rivet.pegboard.kv.Request.Get({
							keys: [
								new rivet.pegboard.kv.Key({
									segments: [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])]
								})
							]
						})
					})
				});
				client.write(encodeFrame(kvMessage2));
			} else if (packet.signal_actor) {
				const response = new protocol.ToManager({
					actor_state_update: new protocol.ToManager.ActorStateUpdate({
						actor_id: packet.signal_actor.actor_id,
						generation: packet.signal_actor.generation,
						state: new protocol.ActorState({
							exited: new protocol.ActorState.Exited({
								exit_code: 0
							})
						})
					})
				});
				client.write(encodeFrame(response));
			}
		}
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

