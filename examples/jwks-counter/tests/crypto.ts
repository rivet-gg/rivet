import assert from "node:assert/strict";
import { createHash, generateKeyPairSync } from "node:crypto";
import { decodeProtectedHeader, importJWK, jwtVerify } from "jose";
import {
	audience,
	issuer,
	publicJwks,
	signCounterToken,
} from "../src/issuer.ts";

const { keys } = publicJwks();
assert.equal(keys.length >= 1, true);
const publicJwk = keys[0];
if (!publicJwk) throw new Error("missing public signing key");
assert.equal("d" in publicJwk, false, "never publish a private JWK component");
const key = await importJWK(publicJwk, "EdDSA");
const token = await signCounterToken();
assert.equal(decodeProtectedHeader(token).typ, "rivet-auth+jwt");
assert.equal(decodeProtectedHeader(token).kid, keys[0]?.kid);
assert.deepEqual(Object.keys(decodeProtectedHeader(token)).sort(), [
	"alg",
	"kid",
	"typ",
]);
const verified = await jwtVerify(token, key, {
	issuer,
	audience,
	algorithms: ["EdDSA"],
});
assert.equal(verified.payload.rivet_ver, 1);
assert.equal(typeof verified.payload.rivet_grants, "string");
assert.deepEqual(Object.keys(verified.payload).sort(), [
	"aud",
	"exp",
	"iat",
	"iss",
	"jti",
	"rivet_grants",
	"rivet_ns",
	"rivet_ver",
	"sub",
]);

const nextKey = generateKeyPairSync("ed25519").publicKey.export({
	format: "jwk",
});
if (!nextKey.x) throw new Error("missing generated Ed25519 public component");
const nextKid = createHash("sha256")
	.update(nextKey.x)
	.digest()
	.subarray(0, 16)
	.toString("base64url");
process.env.JWKS_ADDITIONAL_PUBLIC_KEYS = JSON.stringify([
	{ ...nextKey, kid: nextKid, use: "sig", alg: "EdDSA" },
]);
assert.equal(
	publicJwks().keys.length,
	2,
	"stage a public key before rotating the signer",
);
process.env.JWKS_ADDITIONAL_PUBLIC_KEYS = JSON.stringify([
	{ ...nextKey, kid: nextKid, use: "sig", alg: "EdDSA", d: "private" },
]);
assert.throws(() => publicJwks(), /public Ed25519 JWKs/);
delete process.env.JWKS_ADDITIONAL_PUBLIC_KEYS;
console.log("External JWT signing and public-only JWKS verification passed");
