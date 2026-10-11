import assert from "node:assert/strict";
import { encodeActorGatewayReadGrant } from "../src/grants.ts";

const any = Buffer.from(encodeActorGatewayReadGrant("any"), "base64url");
assert.deepEqual([...any], [1, 0, 1, 7, 0, 1, 0]);

// Rivet's version-1 nil ID is a 1 followed by 29 zeroes in its little-endian base36 form.
const id = `1${"0".repeat(29)}`;
const scoped = Buffer.from(encodeActorGatewayReadGrant({ id }), "base64url");
assert.deepEqual(
	[...scoped],
	[1, 0, 1, 7, 1, 19, 1, ...Array(18).fill(0), 1, 0],
);
assert.equal(
	encodeActorGatewayReadGrant({ id: id.toUpperCase() }),
	encodeActorGatewayReadGrant({ id }),
);
assert.throws(() => encodeActorGatewayReadGrant({ id: "not-a-rivet-id" }));

console.log("JWKS grant encoding passed");
