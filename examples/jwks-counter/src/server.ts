import {
	createServer,
	type IncomingMessage,
	type ServerResponse,
} from "node:http";
import { registry } from "./counter.ts";
import {
	actorId,
	authenticate,
	publicJwks,
	signCounterToken,
} from "./issuer.ts";

export async function handleRequest(
	req: IncomingMessage,
	res: ServerResponse,
): Promise<void> {
	res.setHeader("Content-Type", "application/json");
	if (req.method === "GET" && req.url === "/.well-known/jwks.json") {
		try {
			const jwks = publicJwks();
			res.setHeader("Cache-Control", "public, max-age=30");
			res.writeHead(200).end(JSON.stringify(jwks));
		} catch {
			res.setHeader("Cache-Control", "no-store");
			res.writeHead(503).end(
				JSON.stringify({ error: "issuer_unavailable" }),
			);
		}
		return;
	}
	res.setHeader("Cache-Control", "no-store");
	if (req.method !== "POST" || req.url !== "/token") {
		res.writeHead(404).end(JSON.stringify({ error: "not_found" }));
		return;
	}
	if (!authenticate(req.headers.authorization)) {
		res.writeHead(401, {
			"WWW-Authenticate": 'Basic realm="JWKS counter"',
		}).end(JSON.stringify({ error: "unauthorized" }));
		return;
	}
	try {
		const token = await signCounterToken();
		res.writeHead(200).end(JSON.stringify({ actorId, token }));
	} catch {
		res.writeHead(503).end(JSON.stringify({ error: "issuer_unavailable" }));
	}
}

if (import.meta.url === `file://${process.argv[1]}`) {
	registry.start();
	const host =
		process.argv.indexOf("--host") >= 0
			? process.argv[process.argv.indexOf("--host") + 1]
			: "127.0.0.1";
	createServer((req, res) => {
		void handleRequest(req, res);
	}).listen(3021, host);
}
