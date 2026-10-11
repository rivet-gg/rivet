import assert from "node:assert/strict";
import { once } from "node:events";
import { createServer } from "node:http";
import { decodeJwt } from "jose";
import { handleRequest } from "../src/server.ts";

const server = createServer((req, res) => {
	void handleRequest(req, res);
});
server.listen(0, "127.0.0.1");
await once(server, "listening");
try {
	const address = server.address();
	if (!address || typeof address === "string")
		throw new Error("missing test listener");
	const base = `http://127.0.0.1:${address.port}`;
	const jwks = await fetch(`${base}/.well-known/jwks.json`);
	assert.equal(jwks.status, 200);
	assert.equal(jwks.headers.get("cache-control"), "public, max-age=30");
	const { keys } = (await jwks.json()) as { keys: Record<string, unknown>[] };
	assert.equal(keys.length, 1);
	const publicJwk = keys[0];
	if (!publicJwk) throw new Error("missing public signing key");
	assert.equal("d" in publicJwk, false);
	const authorized = () =>
		fetch(`${base}/token`, {
			method: "POST",
			headers: {
				Authorization: `Basic ${Buffer.from(`${process.env.DEMO_USER}:${process.env.DEMO_PASSWORD}`).toString("base64")}`,
			},
		});
	process.env.JWKS_ADDITIONAL_PUBLIC_KEYS = "not-json";
	try {
		const unavailable = await fetch(`${base}/.well-known/jwks.json`);
		assert.equal(unavailable.status, 503);
		assert.equal(unavailable.headers.get("cache-control"), "no-store");
		const unavailableToken = await authorized();
		assert.equal(unavailableToken.status, 503);
	} finally {
		delete process.env.JWKS_ADDITIONAL_PUBLIC_KEYS;
	}
	assert.equal(
		(await fetch(`${base}/token`, { method: "POST" })).status,
		401,
	);
	const first = await authorized();
	assert.equal(first.status, 200);
	assert.equal(first.headers.get("cache-control"), "no-store");
	const second = await authorized();
	const firstToken = ((await first.json()) as { token: string }).token;
	const secondToken = ((await second.json()) as { token: string }).token;
	assert.notEqual(decodeJwt(firstToken).jti, decodeJwt(secondToken).jti);
	console.log(
		"JWKS issuer HTTP test passed: public-only JWKS and authenticated token renewal",
	);
} finally {
	server.close();
}
