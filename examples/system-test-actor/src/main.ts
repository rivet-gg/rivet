import { serve } from "@hono/node-server";
import { createNodeWebSocket } from "@hono/node-ws";
import { createAndStartHttpServer } from "./httpServer.js";
import { createAndStartUdpServer } from "./udpServer.js";
import { connectToManager } from "./managerClient.js";

let injectWebSocket: any;
const { app, port } = createAndStartHttpServer((app) => {
	// Get Node.js WebSocket handler
	const result = createNodeWebSocket({ app });
	injectWebSocket = result.injectWebSocket;
	return result.upgradeWebSocket;
});

const server = serve({ fetch: app.fetch, port });
injectWebSocket(server);

createAndStartUdpServer();

if (process.env.MULTI) {
	connectToManager();
}
