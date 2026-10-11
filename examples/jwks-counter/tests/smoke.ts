import assert from "node:assert/strict";
import { randomUUID } from "node:crypto";
import { createClient } from "rivetkit/client";
import { counterClient } from "../src/client.ts";
import { endpoint, required } from "../src/config.ts";
import type { registry } from "../src/counter.ts";
import { actorId, audience, issuer, signCounterToken } from "../src/issuer.ts";

const engine = endpoint();
const namespace = required("RIVET_NAMESPACE");
const expected = await counterClient();
assert.equal(expected.incremented, expected.initial + 1);
assert.equal(expected.current, expected.incremented);

async function expectRejected(token: string, code: string): Promise<void> {
	const client = createClient<typeof registry>({
		endpoint: engine,
		namespace,
		disableMetadataLookup: true,
		getToken: async () => token,
	});
	await assert.rejects(
		client.counter.getForId(actorId).getCount(),
		(error: unknown) => {
			return (
				error instanceof Error &&
				"group" in error &&
				error.group === "auth" &&
				"code" in error &&
				error.code === code
			);
		},
	);
}

const now = Math.floor(Date.now() / 1_000);
await expectRejected(
	await signCounterToken({ issuedAt: now - 120, expiresAt: now - 60 }),
	"token_expired",
);
await expectRejected(
	await signCounterToken({ audience: "wrong-audience" }),
	"invalid_token",
);
await expectRejected(
	await signCounterToken({ namespaceId: randomUUID() }),
	"invalid_token",
);
await expectRejected(
	await signCounterToken({ grants: required("JWKS_EXCESS_GRANTS") }),
	"invalid_token",
);

// This destructive test is opt-in, dedicated to a disposable namespace, and
// restores registration even if an assertion fails. Never use it in production.
if (process.env.JWKS_TEST_REMOVE_ISSUER === "1") {
	if (!namespace.startsWith("jwks-smoke-"))
		throw new Error(
			"Use a dedicated jwks-smoke-* namespace for issuer deletion tests",
		);
	const token = required("RIVET_PROVISION_TOKEN");
	const url = new URL(
		`/namespaces/${encodeURIComponent(namespace)}/auth/issuers/counter-demo`,
		engine,
	);
	const body = {
		issuer,
		audience,
		jwks_url: required("JWKS_PUBLIC_URL"),
		algorithm: "EdDSA",
		grant_ceiling: [
			{
				resource: "actor_gateway",
				target: { id: actorId },
				operations: ["read"],
			},
		],
	};
	try {
		const remove = await fetch(url, {
			method: "DELETE",
			headers: { Authorization: `Bearer ${token}` },
		});
		assert.equal(remove.ok, true);
		// The verifier caches external registrations for at most 30 seconds.
		await new Promise((resolve) => setTimeout(resolve, 32_000));
		await expectRejected(await signCounterToken(), "invalid_token");
	} finally {
		const restore = await fetch(url, {
			method: "PUT",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
			},
			body: JSON.stringify(body),
		});
		assert.equal(
			restore.ok,
			true,
			"restore issuer registration before exiting",
		);
	}
}
console.log(
	"JWKS counter smoke passed: scoped access and issuer/audience/expiry/grant isolation",
);
