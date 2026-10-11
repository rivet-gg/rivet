import { createClient } from "rivetkit/client";
import { endpoint, required } from "./config.ts";
import type { registry } from "./counter.ts";
import { encodeActorGatewayReadGrant } from "./grants.ts";

const engine = endpoint();
const namespace = required("RIVET_NAMESPACE");
const namespaceId = required("RIVET_NAMESPACE_ID");
// ACL management is used only during provisioning, not to sign end-user JWTs.
const token = required("RIVET_PROVISION_TOKEN");
const issuer = required("JWKS_ISSUER");
const jwksUrl = required("JWKS_PUBLIC_URL");
const audience = "rivet-api";
const issuerName = "counter-demo";

const manager = createClient<typeof registry>({
	endpoint: engine,
	namespace,
	token,
	disableMetadataLookup: true,
});
const actorId = await manager.counter
	.getOrCreate(["user", required("DEMO_USER")])
	.resolve();

const scoped = encodeActorGatewayReadGrant({ id: actorId });
const excess = encodeActorGatewayReadGrant("any");
const registration = {
	issuer,
	audience,
	jwks_url: jwksUrl,
	algorithm: "EdDSA",
	grant_ceiling: [
		{
			resource: "actor_gateway",
			target: { id: actorId },
			operations: ["read"],
		},
	],
};
const url = new URL(
	`/namespaces/${encodeURIComponent(namespace)}/auth/issuers/${issuerName}`,
	engine,
);
const registered = await fetch(url, {
	method: "PUT",
	headers: {
		Authorization: `Bearer ${token}`,
		"Content-Type": "application/json",
	},
	body: JSON.stringify(registration),
});
if (!registered.ok)
	throw new Error(`issuer registration failed: HTTP ${registered.status}`);

// None of these fields is a credential without the private signing key. Never
// print the ACL token or private key itself.
console.log(
	JSON.stringify(
		{
			JWKS_ACTOR_ID: actorId,
			JWKS_NAMESPACE_ID: namespaceId,
			JWKS_GRANTS: scoped,
			JWKS_EXCESS_GRANTS: excess,
			issuer,
			audience,
			jwksUrl,
		},
		null,
		2,
	),
);
