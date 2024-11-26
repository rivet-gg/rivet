import { Context as HonoContext, Hono } from "hono";
import { cors } from "hono/cors";
import { RivetClientClient } from "@rivet-gg/api";
import { queryActor } from "./query_exec.ts";
import { assertExists } from "@std/assert/exists";
import { assertUnreachable, PORT_NAME, RivetEnvironment } from "../../common/src/utils.ts";
import { ActorsRequest, ActorsResponse } from "../../manager-protocol/src/mod.ts";

const portStr = Deno.env.get("PORT_http") ?? Deno.env.get("HTTP_PORT");
if (!portStr) throw "Missing port";
const port = parseInt(portStr);
if (!isFinite(port)) throw "Invalid port";

// TODO: api endpoint & service token
// TODO: Get rid of this ugly garbage
const TOKEN =
	"env_svc.eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.CPy6y_yYQBD8ksXhtjIaEgoQ7vu2pTW9Sh-klx_juRpufSIXmgEUChIKEOaWqUrqXkn7iZjUQS58f40.gmHt4P3wSbfDY7gbGxvEdp4xW-SzFSFABUoOJBlxrv9WnnE_vWhwYIbgHRmYHTsNrbYiWZaOWpt4PVi9AdNcAA";
const rivetClient = new RivetClientClient({
	environment: "http://rivet-server:8080",
	// TODO: Allow not providing token in dev
	token: TOKEN,
});

// TODO: Read environment
const environment: RivetEnvironment = {};

const app = new Hono();

app.use("/*", cors());

app.post("/actors", async (c: HonoContext) => {
	// Get actor
	const body: ActorsRequest = await c.req.json();
	const actor = await queryActor(rivetClient, environment, body.query);

	// Fetch port
	const httpPort = actor.network.ports[PORT_NAME];
	assertExists(httpPort, "missing port");
	const hostname = httpPort.publicHostname;
	assertExists(hostname);
	const port = httpPort.publicPort;
	assertExists(port);

	let isTls: boolean = false;
	switch (httpPort.protocol) {
		case "https":
			isTls = true;
			break;
		case "http":
		case "tcp":
			isTls = false;
			break;
		case "tcp_tls":
		case "udp":
			throw new Error(`Invalid protocol ${httpPort.protocol}`);
		default:
			assertUnreachable(httpPort.protocol);
	}

	const endpoint = `${isTls ? "https" : "http"}://${hostname}:${port}`;

	return c.json({ endpoint } satisfies ActorsResponse);
});

Deno.serve({
	port,
	hostname: "0.0.0.0",
}, app.fetch);
