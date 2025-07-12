import WebSocket from "ws";

export function connectToManager() {
	let managerIp = process.env.RIVET_MANAGER_IP;
	let managerPort = process.env.RIVET_MANAGER_PORT;
	let pingInterval: NodeJS.Timeout;

	if (!managerIp || !managerPort) {
		console.error("Missing RIVET_MANAGER_IP or RIVET_MANAGER_PORT environment variables");
		return;
	}

	let wsUrl = `ws://${managerIp}:${managerPort}`;
	console.log(`Connecting to manager WebSocket at ${wsUrl}`);

	let ws = new WebSocket(wsUrl);

	ws.on("open", () => {
		console.log("Connected to manager WebSocket");

		let message = {
			init: {
				runner_id: process.env.RIVET_RUNNER_ID
			}
		};
		let buffer = Buffer.from(JSON.stringify(message));
		ws.send(buffer);

		// Start ping loop to keep connection alive
		pingInterval = setInterval(() => {
			if (ws.readyState === WebSocket.OPEN) {
				ws.ping();
			}
		}, 2000);
	});

	ws.on("message", (data) => {
		let json = data.toString();

		console.log("Received message from manager:", json);

		let packet = JSON.parse(json);

		if (packet.start_actor) {
			let message = {
				actor_state_update: {
					actor_id: packet.start_actor.actor_id,
					generation: packet.start_actor.generation,
					state: {
						running: null,
					},
				}
			};
			let buffer = Buffer.from(JSON.stringify(message));
			ws.send(buffer);
		} else if (packet.signal_actor) {
			let message = {
				actor_state_update: {
					actor_id: packet.start_actor.actor_id,
					generation: packet.start_actor.generation,
					state: {
						exited: {
							exit_code: 0,
						}
					},
				}
			};
			let buffer = Buffer.from(JSON.stringify(message));
			ws.send(buffer);
		}
	});

	ws.on("error", (error) => {
		console.error("WebSocket error:", error);
	});

	ws.on("close", code => {
		console.log("WebSocket connection closed, attempting to reconnect...", code);

		// Clear ping interval when connection closes
		if (pingInterval) clearInterval(pingInterval);

		setTimeout(connectToManager, 5000);
	});
}
