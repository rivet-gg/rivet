//! Verifies explicitly registered, namespace-bound externally signed JWTs.
//!
//! Unverified `iss` and `rivet_ns` are lookup hints only. Neither grants access until the
//! signature, exact issuer/audience, bounded expiry, signed grants, and configured grant ceiling
//! have all been checked. This path never falls back to ACL.

use std::{
	collections::BTreeMap,
	future::Future,
	net::{IpAddr, SocketAddr},
	pin::Pin,
	str::FromStr,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::{Duration, Instant},
};

use anyhow::{Context, Result, bail, ensure};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, jwk::Jwk};
use rivet_auth_policy::can_delegate;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
	Claims, DecodedToken, ExternalIssuerRegistry, Id, PROTOCOL_CLOCK_SKEW, TokenError, VerifiedJwt,
	codec::validate_claims,
};

use super::{KeyRingCache, VerificationFailure};

const CACHE_TTL: Duration = Duration::from_secs(30);
const UNKNOWN_KID_COOLDOWN: Duration = Duration::from_secs(5);
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_CACHED_ISSUERS: usize = 128;
const MAX_COLD_ISSUERS: usize = 16;
const FAILED_REFRESH_COOLDOWN: Duration = Duration::from_secs(5);
const MAX_JWKS_BYTES: usize = 32 * 1024;
const MAX_JWKS_KEYS: usize = 16;
const MAX_EXTERNAL_TOKEN_BYTES: usize = 8 * 1024;

type RegistryFuture = Pin<Box<dyn Future<Output = Result<ExternalIssuerRegistry>> + Send>>;
pub(super) type RegistryFetcher = Arc<dyn Fn(Id) -> RegistryFuture + Send + Sync>;
type JwksFuture = Pin<Box<dyn Future<Output = Result<BTreeMap<String, DecodingKey>>> + Send>>;
pub(super) type JwksFetcher = Arc<dyn Fn(crate::ExternalIssuer) -> JwksFuture + Send + Sync>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictHeader {
	typ: String,
	alg: String,
	kid: String,
}

#[derive(Deserialize)]
struct JwksDocument {
	// RFC 7517 allows additional JWK Set members and requires readers to ignore them.
	keys: Vec<serde_json::Value>,
}

pub(super) struct ExternalIssuerCache {
	pub(super) fetched_at: Instant,
	pub(super) last_unknown_kid: Option<Instant>,
	pub(super) registry: ExternalIssuerRegistry,
	pub(super) keys: BTreeMap<(String, String), DecodingKey>,
}

pub(super) struct ExternalIssuerEntry {
	pub(super) state: tokio::sync::Mutex<ExternalIssuerState>,
	first_seen: Instant,
	verified: AtomicBool,
}

#[derive(Default)]
pub(super) struct ExternalIssuerState {
	pub(super) cached: Option<ExternalIssuerCache>,
	pub(super) last_failed_refresh: Option<Instant>,
	last_cold_result: Option<Instant>,
}

impl ExternalIssuerEntry {
	fn new() -> Self {
		Self {
			state: tokio::sync::Mutex::new(ExternalIssuerState::default()),
			first_seen: Instant::now(),
			verified: AtomicBool::new(false),
		}
	}
}

pub(super) fn unverified_claims(token: &str) -> Option<Claims> {
	if token.len() > MAX_EXTERNAL_TOKEN_BYTES {
		return None;
	}
	let (_, remainder) = token.split_once('.')?;
	let (payload, _) = remainder.split_once('.')?;
	if payload.len() > 6_144 {
		return None;
	}
	let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
	serde_json::from_slice(&decoded).ok()
}

fn header(token: &str) -> Result<StrictHeader, VerificationFailure> {
	if token.len() > MAX_EXTERNAL_TOKEN_BYTES {
		return Err(VerificationFailure::InvalidToken);
	}
	let (header, _) = token
		.split_once('.')
		.ok_or(VerificationFailure::InvalidToken)?;
	if header.len() > 512 {
		return Err(VerificationFailure::InvalidToken);
	}
	let bytes = URL_SAFE_NO_PAD
		.decode(header)
		.map_err(|_| VerificationFailure::InvalidToken)?;
	let header: StrictHeader =
		serde_json::from_slice(&bytes).map_err(|_| VerificationFailure::InvalidToken)?;
	if header.typ != crate::TOKEN_TYPE
		|| header.kid.is_empty()
		|| header.kid.len() > 128
		|| !header.kid.is_ascii()
	{
		return Err(VerificationFailure::InvalidToken);
	}
	Ok(header)
}

impl KeyRingCache {
	pub(super) async fn verify_external(
		&self,
		token: &str,
		unverified: Claims,
	) -> Result<VerifiedJwt, VerificationFailure> {
		let header = header(token)?;
		let namespace_id =
			Id::from_str(&unverified.rivet_ns).map_err(|_| VerificationFailure::InvalidToken)?;
		if unverified.iss.len() > 512 {
			return Err(VerificationFailure::InvalidToken);
		}
		let cache_key = (namespace_id, unverified.iss.clone());
		// A lock per issuer coalesces its refreshes without letting an unavailable issuer
		// block verification using another issuer's already cached keys.
		let entry = self.external_entry(&cache_key).await?;
		let mut state = entry.state.lock().await;
		let needs_refresh = state
			.cached
			.as_ref()
			.is_none_or(|entry| entry.fetched_at.elapsed() > CACHE_TTL);
		if needs_refresh {
			self.refresh_external_entry(&mut state, &cache_key).await?;
		}
		let issuer_name = {
			let cached = state
				.cached
				.as_ref()
				.ok_or(VerificationFailure::InvalidToken)?;
			cached
				.registry
				.issuers
				.iter()
				.find(|entry| entry.issuer == unverified.iss)
				.map(|entry| entry.name.clone())
		};
		let Some(issuer_name) = issuer_name else {
			entry.verified.store(false, Ordering::Release);
			return Err(VerificationFailure::InvalidToken);
		};
		let key = (issuer_name.clone(), header.kid.clone());
		if !state
			.cached
			.as_ref()
			.is_some_and(|entry| entry.keys.contains_key(&key))
		{
			let last_unknown = state
				.cached
				.as_ref()
				.and_then(|entry| entry.last_unknown_kid);
			if last_unknown.is_none_or(|when| when.elapsed() >= UNKNOWN_KID_COOLDOWN) {
				if let Some(entry) = state.cached.as_mut() {
					entry.last_unknown_kid = Some(Instant::now());
				}
				self.refresh_external_entry(&mut state, &cache_key).await?;
			}
		}
		let cached = state
			.cached
			.as_ref()
			.ok_or(VerificationFailure::InvalidToken)?;
		let issuer = cached
			.registry
			.issuers
			.iter()
			.find(|entry| entry.name == issuer_name && entry.issuer == unverified.iss);
		let Some(issuer) = issuer else {
			entry.verified.store(false, Ordering::Release);
			return Err(VerificationFailure::InvalidToken);
		};
		if header.alg != issuer.algorithm {
			return Err(VerificationFailure::InvalidToken);
		}
		let decoding_key = cached
			.keys
			.get(&key)
			.ok_or(VerificationFailure::InvalidToken)?;
		let algorithm = match issuer.algorithm.as_str() {
			"EdDSA" => Algorithm::EdDSA,
			"RS256" => Algorithm::RS256,
			"ES256" => Algorithm::ES256,
			_ => return Err(VerificationFailure::InvalidToken),
		};
		let mut validation = Validation::new(algorithm);
		validation.required_spec_claims.clear();
		validation.validate_exp = false;
		validation.validate_aud = false;
		validation.leeway = 0;
		let decoded = jsonwebtoken::decode::<Claims>(token, decoding_key, &validation)
			.map_err(|_| VerificationFailure::InvalidToken)?;
		let now = u64::try_from(rivet_util::timestamp::now() / 1000)
			.map_err(|_| VerificationFailure::VerificationUnavailable)?;
		let (token_namespace, grants) = validate_claims(
			&decoded.claims,
			Some(crate::DecodeOptions {
				issuer: &issuer.issuer,
				audience: &issuer.audience,
				now,
			}),
		)
		.map_err(|error| match error {
			TokenError::Expired => VerificationFailure::Expired,
			_ => VerificationFailure::InvalidToken,
		})?;
		if token_namespace != namespace_id
			|| decoded.claims.sub.as_deref().is_none_or(str::is_empty)
			|| !can_delegate(&issuer.grant_ceiling, &grants)
		{
			return Err(VerificationFailure::InvalidToken);
		}
		// External `kid` is opaque. A deterministic, issuer-qualified fingerprint serves as
		// internal diagnostic metadata without conflating it with a Rivet signing key ID.
		let digest =
			Sha256::digest([issuer.issuer.as_bytes(), b":", header.kid.as_bytes()].concat());
		let mut kid_fingerprint = [0; crate::keys::KEY_ID_BYTES];
		kid_fingerprint.copy_from_slice(&digest[..crate::keys::KEY_ID_BYTES]);
		entry.verified.store(true, Ordering::Release);
		self.trim_verified_cache(&cache_key, &entry).await;
		Ok(VerifiedJwt {
			authorization_deadline: decoded.claims.exp.saturating_add(PROTOCOL_CLOCK_SKEW),
			token: DecodedToken {
				claims: decoded.claims,
				namespace_id,
				grants,
			},
			kid: crate::KeyId::from_bytes(kid_fingerprint),
		})
	}

	async fn external_entry(
		&self,
		cache_key: &(Id, String),
	) -> Result<Arc<ExternalIssuerEntry>, VerificationFailure> {
		let entry = {
			let mut caches = self.external_cache.lock().await;
			if let Some(entry) = caches.get(&cache_key) {
				Arc::clone(entry)
			} else {
				let cold_count = caches
					.values()
					.filter(|entry| !entry.verified.load(Ordering::Acquire))
					.count();
				if cold_count >= MAX_COLD_ISSUERS {
					// Unverified hints may consume a small cold budget, but cannot evict an
					// in-flight refresh or a verified issuer's warm keys. A newly verified
					// issuer may replace an old verified entry only after validation.
					let expired = caches
						.iter()
						.find(|(_, entry)| {
							!entry.verified.load(Ordering::Acquire)
								// The map must be the sole owner. A caller can hold an Arc before it
								// acquires the state lock, so try_lock alone cannot protect it.
								&& Arc::strong_count(entry) == 1
								&& entry.state.try_lock().is_ok_and(|state| {
									// A cancelled first refresh never records a result. Once its caller
									// drops the Arc, admission time still lets us reclaim the slot.
									state
										.last_cold_result
										.unwrap_or(entry.first_seen)
										.elapsed()
										>= FAILED_REFRESH_COOLDOWN
								})
						})
						.map(|(key, _)| key.clone());
					if let Some(expired) = expired {
						caches.remove(&expired);
					} else {
						return Err(VerificationFailure::VerificationUnavailable);
					}
				}
				let entry = Arc::new(ExternalIssuerEntry::new());
				caches.insert(cache_key.clone(), Arc::clone(&entry));
				entry
			}
		};
		Ok(entry)
	}

	async fn trim_verified_cache(
		&self,
		cache_key: &(Id, String),
		entry: &Arc<ExternalIssuerEntry>,
	) {
		let mut caches = self.external_cache.lock().await;
		if !caches
			.get(cache_key)
			.is_some_and(|current| Arc::ptr_eq(current, entry))
		{
			return;
		}
		let verified_count = caches
			.values()
			.filter(|entry| entry.verified.load(Ordering::Acquire))
			.count();
		if verified_count > MAX_CACHED_ISSUERS {
			let evicted = caches
				.iter()
				.find(|(key, entry)| *key != cache_key && entry.verified.load(Ordering::Acquire))
				.map(|(key, _)| key.clone());
			if let Some(evicted) = evicted {
				caches.remove(&evicted);
			}
		}
	}

	async fn refresh_external_entry(
		&self,
		state: &mut ExternalIssuerState,
		cache_key: &(Id, String),
	) -> Result<(), VerificationFailure> {
		if state
			.last_failed_refresh
			.is_some_and(|when| when.elapsed() < FAILED_REFRESH_COOLDOWN)
		{
			return self.stale_external(&state.cached);
		}
		let result = self.refresh_external(&mut state.cached, cache_key).await;
		let completed_at = Instant::now();
		state.last_cold_result = Some(completed_at);
		match result {
			Ok(true) => state.last_failed_refresh = None,
			Ok(false) => state.last_failed_refresh = Some(completed_at),
			Err(error) => {
				state.last_failed_refresh = Some(completed_at);
				return Err(error);
			}
		}
		Ok(())
	}

	async fn refresh_external(
		&self,
		cached: &mut Option<ExternalIssuerCache>,
		cache_key: &(Id, String),
	) -> Result<bool, VerificationFailure> {
		let refreshed = async {
			// Epoxy waits up to REQUEST_TIMEOUT for a blackholed replica before it can
			// select the newest value from the replicas that did answer. Do not cut
			// that read short with the smaller outbound JWKS HTTP timeout.
			let registry = tokio::time::timeout(
				epoxy::consts::REQUEST_TIMEOUT + FETCH_TIMEOUT,
				(self.external_fetch_registry)(cache_key.0),
			)
			.await??;
			// Prevent rollback within this process. An empty/evicted cache has no generation floor:
			// a stale reachable replica can delay issuer revocation until a current replica returns.
			// max_stale only bounds fetch failures, not successful reads of stale committed data.
			ensure!(
				cached
					.as_ref()
					.is_none_or(|previous| registry.generation >= previous.registry.generation),
				"JWT issuer registry generation regressed"
			);
			let mut keys = BTreeMap::new();
			for issuer in registry
				.issuers
				.iter()
				.filter(|entry| entry.issuer == cache_key.1)
			{
				for (kid, key) in
					tokio::time::timeout(FETCH_TIMEOUT, (self.external_fetch_jwks)(issuer.clone()))
						.await??
				{
					ensure!(
						keys.insert((issuer.name.clone(), kid), key).is_none(),
						"duplicate issuer kid"
					);
				}
			}
			Ok::<_, anyhow::Error>(ExternalIssuerCache {
				fetched_at: Instant::now(),
				last_unknown_kid: cached
					.as_ref()
					.and_then(|previous| previous.last_unknown_kid),
				registry,
				keys,
			})
		}
		.await;
		match refreshed {
			Ok(entry) => {
				*cached = Some(entry);
				Ok(true)
			}
			Err(error) => {
				tracing::warn!(?error, "external JWT verifier refresh failed");
				self.stale_external(cached)?;
				Ok(false)
			}
		}
	}

	fn stale_external(
		&self,
		cached: &Option<ExternalIssuerCache>,
	) -> Result<(), VerificationFailure> {
		if cached
			.as_ref()
			.is_some_and(|entry| entry.fetched_at.elapsed() <= self.max_stale)
		{
			Ok(())
		} else {
			Err(VerificationFailure::VerificationUnavailable)
		}
	}
}

pub(super) async fn fetch_jwks(
	issuer: &crate::ExternalIssuer,
) -> Result<BTreeMap<String, DecodingKey>> {
	let url = reqwest::Url::parse(&issuer.jwks_url)?;
	ensure!(
		url.scheme() == "https" && url.port_or_known_default() == Some(443),
		"JWKS URL must use HTTPS port 443"
	);
	let host = url.host_str().context("JWKS URL is missing a host")?;
	let addresses = tokio::net::lookup_host((host, 443))
		.await?
		.collect::<Vec<SocketAddr>>();
	ensure!(
		!addresses.is_empty() && addresses.iter().all(|addr| public_ip(addr.ip())),
		"JWKS URL resolved outside the public internet"
	);
	// Pin this particular fetch to the validated DNS answers. Disabling redirects also blocks a
	// public endpoint from redirecting to a private address or a different issuer.
	let client = jwks_client(host, &addresses)?;
	let mut response = client.get(url).send().await?.error_for_status()?;
	ensure!(response.status().is_success(), "JWKS fetch did not succeed");
	let mut bytes = Vec::new();
	while let Some(chunk) = response.chunk().await? {
		ensure!(
			bytes.len().saturating_add(chunk.len()) <= MAX_JWKS_BYTES,
			"JWKS document too large"
		);
		bytes.extend_from_slice(&chunk);
	}
	parse_jwks(&bytes, &issuer.algorithm)
}

fn jwks_client(host: &str, addresses: &[SocketAddr]) -> Result<reqwest::Client> {
	Ok(reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.no_proxy()
		.resolve_to_addrs(host, addresses)
		.timeout(FETCH_TIMEOUT)
		.build()?)
}

fn parse_jwks(bytes: &[u8], algorithm: &str) -> Result<BTreeMap<String, DecodingKey>> {
	ensure!(bytes.len() <= MAX_JWKS_BYTES, "JWKS document too large");
	let document: JwksDocument = serde_json::from_slice(bytes)?;
	ensure!(
		document.keys.len() <= MAX_JWKS_KEYS,
		"invalid JWKS key count"
	);
	let mut keys = BTreeMap::new();
	for value in document.keys {
		let object = value.as_object().context("JWKS key must be an object")?;
		// Ignore extension metadata as required by RFC 7517, but never accept a JWK that
		// publishes standard private (or symmetric) key components.
		ensure!(
			!object.keys().any(|key| matches!(
				key.as_str(),
				"d" | "p" | "q" | "dp" | "dq" | "qi" | "oth" | "k"
			)),
			"JWKS key contains private material"
		);
		// A key set may publish other algorithms or encryption keys. They cannot verify
		// this issuer's pinned algorithm, so ignore them rather than rejecting the set.
		if object
			.get("alg")
			.is_some_and(|v| v.as_str() != Some(algorithm))
			|| object.get("use").is_some_and(|v| v.as_str() != Some("sig"))
			|| object.get("key_ops").is_some_and(|ops| {
				!ops.as_array()
					.is_some_and(|ops| ops.len() == 1 && ops[0].as_str() == Some("verify"))
			}) {
			continue;
		}
		let compatible_type = match algorithm {
			"EdDSA" => {
				object.get("kty").and_then(|v| v.as_str()) == Some("OKP")
					&& object.get("crv").and_then(|v| v.as_str()) == Some("Ed25519")
			}
			"ES256" => {
				object.get("kty").and_then(|v| v.as_str()) == Some("EC")
					&& object.get("crv").and_then(|v| v.as_str()) == Some("P-256")
			}
			"RS256" => object.get("kty").and_then(|v| v.as_str()) == Some("RSA"),
			_ => bail!("unsupported external signing algorithm"),
		};
		if !compatible_type {
			continue;
		}
		let kid = object
			.get("kid")
			.and_then(|v| v.as_str())
			.context("JWKS key is missing kid")?
			.to_owned();
		ensure!(
			!kid.is_empty() && kid.len() <= 128 && kid.is_ascii(),
			"invalid JWKS key identifier"
		);
		match algorithm {
			"EdDSA" => {
				ensure!(
					jwk_component(object, "x")?.len() == 32,
					"invalid Ed25519 key length"
				);
			}
			"ES256" => {
				ensure!(
					jwk_component(object, "x")?.len() == 32
						&& jwk_component(object, "y")?.len() == 32,
					"invalid P-256 key length"
				);
			}
			"RS256" => {
				ensure!(
					jwk_component(object, "n")?.len() >= 256,
					"RSA signing keys must be at least 2048 bits"
				);
				let exponent = jwk_component(object, "e")?;
				ensure!(
					!exponent.is_empty()
						&& exponent.len() <= 4
						&& exponent.last().is_some_and(|byte| byte & 1 == 1),
					"invalid RSA exponent"
				);
			}
			_ => bail!("unsupported external signing algorithm"),
		}
		let jwk: Jwk = serde_json::from_value(value)?;
		let key = DecodingKey::from_jwk(&jwk)?;
		ensure!(
			keys.insert(kid, key).is_none(),
			"duplicate JWKS key identifier"
		);
	}
	Ok(keys)
}

fn jwk_component(
	object: &serde_json::Map<String, serde_json::Value>,
	name: &str,
) -> Result<Vec<u8>> {
	let encoded = object
		.get(name)
		.and_then(|v| v.as_str())
		.context("missing public JWK component")?;
	ensure!(
		!encoded.is_empty() && encoded.len() <= 1_024 && !encoded.contains('='),
		"invalid public JWK component"
	);
	let bytes = URL_SAFE_NO_PAD.decode(encoded)?;
	ensure!(
		URL_SAFE_NO_PAD.encode(&bytes) == encoded,
		"non-canonical public JWK component"
	);
	Ok(bytes)
}

fn public_ip(ip: IpAddr) -> bool {
	match ip {
		IpAddr::V4(ip) => {
			let [a, b, c, _] = ip.octets();
			// Conservative allowlist of public unicast space; reject private, link-local,
			// documentation, benchmarking, carrier NAT, multicast, and reserved ranges.
			(1..=223).contains(&a)
				&& a != 10 && a != 127
				&& !(a == 100 && (64..=127).contains(&b))
				&& !(a == 169 && b == 254)
				&& !(a == 172 && (16..=31).contains(&b))
				&& !(a == 192 && (b == 168 || b == 0 || (b == 88 && c == 99)))
				&& !(a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
				&& !(a == 203 && b == 0 && c == 113)
		}
		IpAddr::V6(ip) => {
			if let Some(mapped) = ip.to_ipv4_mapped() {
				return public_ip(IpAddr::V4(mapped));
			}
			let segments = ip.segments();
			// IPv6 global-unicast /3, excluding special-purpose and documentation ranges.
			(segments[0] & 0xe000) == 0x2000
				&& !(segments[0] == 0x2001 && segments[1] <= 0x01ff)
				&& !(segments[0] == 0x2001 && segments[1] == 0x0db8)
				&& segments[0] != 0x2002
				&& segments[0] != 0x3fff
		}
	}
}

#[cfg(test)]
#[path = "../../tests/unit/key_ring_cache_external.rs"]
mod tests;
