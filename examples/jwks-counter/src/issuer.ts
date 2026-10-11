import {
	createHash,
	createPrivateKey,
	createPublicKey,
	randomBytes,
	timingSafeEqual,
} from "node:crypto";
import { readFileSync } from "node:fs";
import { exportJWK, importPKCS8, type JWK, SignJWT } from "jose";
import { required } from "./config.ts";

export const issuer = required("JWKS_ISSUER");
export const audience = "rivet-api";
export const actorId = required("JWKS_ACTOR_ID");
export const namespaceId = required("JWKS_NAMESPACE_ID");
export const grants = required("JWKS_GRANTS");
const username = required("DEMO_USER");
const password = required("DEMO_PASSWORD");

// Load private signing material only on the server; never copy it into a client
// bundle or JWKS response. Additional keys must be public-only JWKs.
const pem = readFileSync(required("JWKS_PRIVATE_KEY_PATH"), "utf8");
const publicKey = createPublicKey(createPrivateKey(pem));
const signer = await importPKCS8(pem, "EdDSA");
const currentPublic = await exportJWK(publicKey);
if (
	currentPublic.kty !== "OKP" ||
	currentPublic.crv !== "Ed25519" ||
	!currentPublic.x
) {
	throw new Error("Expected an Ed25519 PKCS#8 signing key");
}
const kid = createHash("sha256")
	.update(currentPublic.x)
	.digest()
	.subarray(0, 16)
	.toString("base64url");

export function publicJwks(): { keys: JWK[] } {
	const additional = JSON.parse(
		process.env.JWKS_ADDITIONAL_PUBLIC_KEYS ?? "[]",
	) as JWK[];
	const allowed = new Set([
		"kty",
		"crv",
		"x",
		"kid",
		"use",
		"alg",
		"key_ops",
	]);
	if (
		!Array.isArray(additional) ||
		additional.length > 15 ||
		additional.some(
			(key) =>
				key === null ||
				typeof key !== "object" ||
				key.kty !== "OKP" ||
				key.crv !== "Ed25519" ||
				typeof key.x !== "string" ||
				Buffer.from(key.x, "base64url").length !== 32 ||
				Buffer.from(key.x, "base64url").toString("base64url") !==
					key.x ||
				typeof key.kid !== "string" ||
				!key.kid ||
				key.kid.length > 128 ||
				!/^[\x21-\x7e]+$/.test(key.kid) ||
				key.alg !== "EdDSA" ||
				key.use !== "sig" ||
				(key.key_ops !== undefined &&
					(!Array.isArray(key.key_ops) ||
						key.key_ops.length !== 1 ||
						key.key_ops[0] !== "verify")) ||
				Object.keys(key).some((field) => !allowed.has(field)),
		) ||
		new Set([kid, ...additional.map((key) => key.kid)]).size !==
			additional.length + 1
	) {
		throw new Error(
			"JWKS_ADDITIONAL_PUBLIC_KEYS must contain at most 15 public Ed25519 JWKs",
		);
	}
	return {
		keys: [
			{ ...currentPublic, kid, use: "sig", alg: "EdDSA" },
			...additional,
		],
	};
}

export function authenticate(header?: string): boolean {
	if (!header) return false;
	const expected = `Basic ${Buffer.from(`${username}:${password}`).toString("base64")}`;
	return timingSafeEqual(
		createHash("sha256").update(header).digest(),
		createHash("sha256").update(expected).digest(),
	);
}

export async function signCounterToken(
	options: {
		grants?: string;
		audience?: string;
		namespaceId?: string;
		issuedAt?: number;
		expiresAt?: number;
	} = {},
): Promise<string> {
	// Never issue a token when its public verification keys cannot be served.
	publicJwks();
	const now = Math.floor(Date.now() / 1_000);
	const issuedAt = options.issuedAt ?? now;
	const expiresAt = options.expiresAt ?? now + 30;
	return new SignJWT({
		rivet_ver: 1,
		rivet_ns: options.namespaceId ?? namespaceId,
		rivet_grants: options.grants ?? grants,
	})
		.setProtectedHeader({ typ: "rivet-auth+jwt", alg: "EdDSA", kid })
		.setIssuer(issuer)
		.setAudience(options.audience ?? audience)
		.setSubject(username)
		.setIssuedAt(issuedAt)
		.setExpirationTime(expiresAt)
		.setJti(randomBytes(16).toString("base64url"))
		.sign(signer);
}
