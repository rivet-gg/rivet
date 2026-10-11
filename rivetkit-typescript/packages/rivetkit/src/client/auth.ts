import type { Rivet } from "@rivetkit/engine-api-full";

/** A namespace-scoped Engine grant. Actor gateway append grants also authorize the primary stream append. */
export type TokenGrant = Rivet.AuthTokenGrant;

/** Options for `client.auth.issueToken()`. `expiresIn` is in seconds. */
export type IssueTokenOptions = Pick<
	Rivet.AuthTokenCreateRequest,
	"subject" | "grants"
> & {
	expiresIn?: Rivet.AuthTokenCreateRequest["duration"];
};

/** Token timestamps are Unix milliseconds. */
export interface IssuedToken {
	token: Rivet.AuthTokenCreateResponse["token"];
	issuedAt: Rivet.AuthTokenCreateResponse["issuedTs"];
	expiresAt: Rivet.AuthTokenCreateResponse["expiresTs"];
}

/** Resources that can be scoped to a single actor ID. */
export type ActorTokenResource = Extract<
	Rivet.AuthTokenGrant["resource"],
	"actor_gateway" | "actor" | "actor_kv" | "actor_stream"
>;

export type ActorTokenPermissions = Partial<
	Record<ActorTokenResource, Rivet.AuthTokenGrant["operations"]>
>;

/** Options for `actor.issueToken()`. Defaults to actor actions and primary stream append. Explicit permissions replace these defaults. */
export type ActorIssueTokenOptions = Pick<
	IssueTokenOptions,
	"subject" | "expiresIn"
> & {
	permissions?: ActorTokenPermissions;
};

/** Pair actor input with primary-stream append, preserving explicit action-only grants. */
export function pairPrimaryStreamGrants(grants: TokenGrant[]): TokenGrant[] {
	const result = [...grants];
	for (const grant of grants) {
		if (
			grant.resource !== "actor_gateway" ||
			!grant.operations.includes("append")
		)
			continue;
		const target = grant.target;
		if (
			result.some(
				(g) =>
					g.resource === "actor_stream" &&
					g.operations.includes("append") &&
					(g.target === target ||
						(typeof g.target === "object" &&
							typeof target === "object" &&
							g.target.id === target.id)),
			)
		)
			continue;
		result.push({
			resource: "actor_stream",
			target,
			operations: ["append"],
		});
	}
	return result;
}
