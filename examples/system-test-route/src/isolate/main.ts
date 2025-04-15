import type { ActorContext } from "@rivet-gg/actor-core";
import { upgradeWebSocket } from "hono/deno";
import { createAndStartServer } from "../shared/server.js";

// Start server
export default {
	async start(ctx: ActorContext) {
		console.log("Isolate starting");

		console.log("Metadata:", ctx.metadata);
		console.log("Actor ID:", ctx.actor_id);
        
		// Store actor ID and selector tags for display
		if (!ctx.actor_id) {
			throw new Error("Missing actor_id in context");
		}
        
		// Create and start server with Deno WebSocket upgrader
		console.log("Starting HTTP server");
		const { app, port } = createAndStartServer(() => upgradeWebSocket, ctx.actor_id);

		const server = Deno.serve(
			{
				port,
			},
			app.fetch,
		);
		await server.finished;
	},
};