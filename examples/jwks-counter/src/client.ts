import { createClient } from "rivetkit/client";
import { endpoint, required } from "./config.ts";
import type { registry } from "./counter.ts";

export async function counterClient() {
	const authorization = `Basic ${Buffer.from(`${required("DEMO_USER")}:${required("DEMO_PASSWORD")}`).toString("base64")}`;
	let actorId: string | undefined;
	const client = createClient<typeof registry>({
		endpoint: endpoint(),
		namespace: required("RIVET_NAMESPACE"),
		disableMetadataLookup: true,
		getToken: async () => {
			const response = await fetch(
				new URL("/token", required("JWKS_ISSUER")),
				{
					method: "POST",
					headers: { Authorization: authorization },
					cache: "no-store",
				},
			);
			if (!response.ok)
				throw new Error(
					`authentication failed: HTTP ${response.status}`,
				);
			const issued = (await response.json()) as {
				token: string;
				actorId: string;
			};
			actorId = issued.actorId;
			return issued.token;
		},
	});
	// The backend returns the ID with the initial signed JWT; the client never
	// gains actor.create or actor.list permissions to discover it itself.
	const first = await client.counter
		.getForId(required("JWKS_ACTOR_ID"))
		.getCount();
	if (!actorId || actorId !== required("JWKS_ACTOR_ID"))
		throw new Error("issuer actor identity mismatch");
	const counter = client.counter.getForId(actorId);
	return {
		initial: first,
		incremented: await counter.increment(1),
		current: await counter.getCount(),
	};
}

if (import.meta.url === `file://${process.argv[1]}`) {
	const result = await counterClient();
	console.log(
		`Counter: ${result.initial} → ${result.incremented} → ${result.current}`,
	);
}
