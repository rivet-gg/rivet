import { actor } from "rivetkit";

export const CONN_DRAIN_DISCONNECT_MARKER = "conn-drain onDisconnect";

export const connDrainActor = actor({
	onDisconnect: async (c) => {
		console.log(`${CONN_DRAIN_DISCONNECT_MARKER} ${c.key.join("/")}`);
		// A slow hook keeps the connection registered while the runtime
		// closes its socket, which is when a duplicate hook can fire.
		await new Promise((resolve) => setTimeout(resolve, 100));
	},
	actions: {
		ping: () => "pong",
	},
});
