# Externally signed JWKS counter

This counter uses RivetKit's renewable `getToken` hook, but the backend signs the JWT with **its own Ed25519 key**. Rivet trusts the exact issuer, audience, public JWKS URL, namespace, algorithm, and maximum grant set registered by a namespace administrator. The backend publishes public verification keys only; clients receive short-lived actor-specific JWTs, never an ACL token or private key.

## Setup

Use an Engine with JWT verification and namespace ACL enabled, and a disposable namespace. The actor host uses its usual server-side RivetKit runtime credentials. Give a **provisioning-only** `RIVET_PROVISION_TOKEN` permission to create/read actors, read `actor_gateway` targets, and update `jwt` issuer registrations in this namespace. It does not need JWT issuance or global namespace-read permission. Keep this token out of the running JWT issuer and all clients.

1. Generate an Ed25519 PKCS#8 key outside the repository, for example `key_dir=$(mktemp -d); openssl genpkey -algorithm Ed25519 -out "$key_dir/private.pem"; export JWKS_PRIVATE_KEY_PATH="$key_dir/private.pem"`. Never commit it.
2. Set `JWKS_ISSUER=https://jwks-counter.gameinc.io` and expose this example server through HTTPS on port 443 at `JWKS_PUBLIC_URL=https://jwks-counter.gameinc.io/.well-known/jwks.json`. The domain must resolve publicly and the URL must not redirect; Engine deliberately rejects localhost, IP literals, redirects, credential-bearing URLs, and non-443 ports. The HTTP server listens on port 3021 behind your TLS proxy/tunnel.
3. On the provisioning machine, set `RIVET_ENDPOINT` (your Engine's HTTPS API endpoint), `RIVET_NAMESPACE` (name), `RIVET_NAMESPACE_ID` (ID from your namespace settings), `RIVET_PROVISION_TOKEN`, `DEMO_USER`, `JWKS_ISSUER`, and `JWKS_PUBLIC_URL`. Run `pnpm --filter jwks-counter-example provision`. This creates the private counter actor, encodes versioned BARE grants with the shared TypeScript data codec, then registers `PUT /namespaces/{namespace}/auth/issuers/counter-demo` with:

   ```json
   {
     "issuer": "https://jwks-counter.gameinc.io",
     "audience": "rivet-api",
     "jwks_url": "https://jwks-counter.gameinc.io/.well-known/jwks.json",
     "algorithm": "EdDSA",
     "grant_ceiling": [{ "resource": "actor_gateway", "target": { "id": "<actor ID>" }, "operations": ["read"] }]
   }
   ```

4. Set the resulting `JWKS_ACTOR_ID`, `JWKS_NAMESPACE_ID`, `JWKS_GRANTS`, and (for negative smoke tests only) `JWKS_EXCESS_GRANTS` in the backend environment. These grant bytes alone cannot authenticate a request without the signing key. Set a strong `DEMO_PASSWORD` for the authenticated single-user demo issuer. **Unset `RIVET_PROVISION_TOKEN` in the runtime backend.** Run `pnpm --filter jwks-counter-example start` and verify that `GET /.well-known/jwks.json` includes the public JWK but no `d` field.
5. Run `pnpm --filter jwks-counter-example client` with `RIVET_ENDPOINT`, `RIVET_NAMESPACE`, `JWKS_ISSUER`, `JWKS_ACTOR_ID`, `DEMO_USER`, and `DEMO_PASSWORD` (no private key, no provisioning token). The client calls `/token` as necessary, then reads and increments its counter through RivetKit. Basic auth is only for this CLI demo: use HTTPS outside loopback and integrate your production user/session authentication for a browser client.

To check signing independently of Engine, run `pnpm --filter jwks-counter-example exec tsx tests/crypto.ts` with the backend issuer environment. With the backend and Engine running, `pnpm --filter jwks-counter-example smoke` verifies valid counter use and rejects expired tokens, wrong audience, wrong namespace, and grants above the ceiling. To also test issuer removal, use **only** a disposable namespace named `jwks-smoke-*` and opt in with `JWKS_TEST_REMOVE_ISSUER=1` plus `RIVET_PROVISION_TOKEN` and `JWKS_PUBLIC_URL`. This deletes the registration, waits for the verifier cache, asserts rejection, and re-registers it in a `finally` block. Do not run the destructive check on a shared namespace.

## Rotating keys

Keep old public keys available until every token they signed expires (30 seconds in this demo, plus 30 seconds of verifier clock skew). To rotate, generate the next key outside Git; first publish its **public-only** JWK in `JWKS_ADDITIONAL_PUBLIC_KEYS` alongside the current key and wait at least one 30-second verifier cache lifetime. Switch `JWKS_PRIVATE_KEY_PATH` to the next private key while retaining the old public JWK in that same public-only list. After the old tokens and skew expire, remove the old public JWK. Never serve a private JWK (`d`) or print the private key. The issuer's EdDSA algorithm and `kid` must match the published JWK.
