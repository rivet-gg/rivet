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

// TOOD: move this cleaner plac
// async function contactApi() {
// 	console.log('Contacting', process.env.RIVET_API_ENDPOINT);
// 	const res = await fetch(process.env.RIVET_API_ENDPOINT!);
// 	console.log('API response', res.ok, res.status);
// }
//
// contactApi();

// TODO: Move this cleaner place
// Print hosts file contents before starting
// try {
// 	const hostsContent = fs.readFileSync('/etc/hosts', 'utf8');
// 	console.log('=== /etc/hosts contents ===');
// 	console.log(hostsContent);
// 	console.log('=== End of /etc/hosts ===');
// } catch (err) {
// 	console.error('Failed to read /etc/hosts:', err);
// }
