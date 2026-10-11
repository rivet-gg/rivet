use super::*;
use rivet_auth_policy::{OperationKind, OwnedGrant, ResourceKind, Scope};
use std::{
	collections::VecDeque,
	sync::{
		Mutex as StdMutex,
		atomic::{AtomicBool, AtomicI64, AtomicU8, AtomicUsize, Ordering},
	},
};
use tokio::sync::{Mutex, watch};

fn verifier_with_registry_source(
	registry_fetcher: RegistryFetcher,
	jwks_reads: Arc<AtomicUsize>,
) -> Arc<KeyRingCache> {
	let (snapshot_tx, _) = watch::channel(None);
	Arc::new(KeyRingCache {
		mode: super::super::KeyRingCacheMode::VerifyOnly,
		fetch_snapshot: Arc::new(|| Box::pin(async { bail!("unused snapshot source") })),
		cache_ttl: CACHE_TTL,
		max_stale: Duration::from_secs(60),
		unknown_kid_cooldown: UNKNOWN_KID_COOLDOWN,
		snapshot_tx,
		refresh_lock: Mutex::new(()),
		background_refresh_running: AtomicBool::new(false),
		last_unknown_refresh_ts: AtomicI64::new(i64::MIN),
		last_refresh_failure_ts: AtomicI64::new(i64::MIN),
		fallback_state: AtomicU8::new(0),
		external_fetch_registry: registry_fetcher,
		external_fetch_jwks: Arc::new(move |_| {
			jwks_reads.fetch_add(1, Ordering::Relaxed);
			Box::pin(async { Ok(BTreeMap::new()) })
		}),
		external_cache: Mutex::new(BTreeMap::new()),
	})
}

fn registry(
	namespace_id: Id,
	generation: u64,
	allowed_operations: Option<Vec<OperationKind>>,
) -> ExternalIssuerRegistry {
	ExternalIssuerRegistry {
		namespace_id,
		generation,
		issuers: allowed_operations
			.map(|operations| {
				vec![crate::ExternalIssuer {
					name: "example".into(),
					issuer: "https://issuer.example.com".into(),
					audience: "rivet-api".into(),
					jwks_url: "https://issuer.example.com/.well-known/jwks.json".into(),
					algorithm: "EdDSA".into(),
					grant_ceiling: vec![OwnedGrant {
						namespace: Scope::Id(namespace_id),
						resource: ResourceKind::Actor,
						target: Scope::Any,
						operations,
					}],
				}]
			})
			.unwrap_or_default(),
	}
}

#[tokio::test]
async fn failed_cold_refresh_is_single_flight_during_cooldown() {
	let reads = Arc::new(AtomicUsize::new(0));
	let registry_fetcher: RegistryFetcher = {
		let reads = Arc::clone(&reads);
		Arc::new(move |_| {
			let reads = Arc::clone(&reads);
			Box::pin(async move {
				reads.fetch_add(1, Ordering::Relaxed);
				bail!("registry unavailable")
			})
		})
	};
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let cache_key = (Id::new_v1(1), "https://issuer.example.com".to_owned());
	let entry = verifier.external_entry(&cache_key).await.unwrap();
	let mut tasks = Vec::new();
	for _ in 0..8 {
		let verifier = Arc::clone(&verifier);
		let entry = Arc::clone(&entry);
		let cache_key = cache_key.clone();
		tasks.push(tokio::spawn(async move {
			let mut state = entry.state.lock().await;
			verifier
				.refresh_external_entry(&mut state, &cache_key)
				.await
		}));
	}
	for task in tasks {
		assert!(matches!(
			task.await.unwrap(),
			Err(VerificationFailure::VerificationUnavailable)
		));
	}
	assert_eq!(reads.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn full_cold_cache_does_not_evict_in_flight_refreshes() {
	let reads = Arc::new(AtomicUsize::new(0));
	let pending = Arc::new(tokio::sync::Semaphore::new(0));
	let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
	let registry_fetcher: RegistryFetcher = {
		let reads = Arc::clone(&reads);
		let pending = Arc::clone(&pending);
		Arc::new(move |_| {
			let reads = Arc::clone(&reads);
			let pending = Arc::clone(&pending);
			let started_tx = started_tx.clone();
			Box::pin(async move {
				reads.fetch_add(1, Ordering::Relaxed);
				started_tx.send(()).unwrap();
				pending.acquire().await.unwrap().forget();
				bail!("registry unavailable")
			})
		})
	};
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let namespace_id = Id::new_v1(1);
	let keys: Vec<_> = (0..MAX_COLD_ISSUERS)
		.map(|index| (namespace_id, format!("https://unknown-{index}.example.com")))
		.collect();
	for key in &keys {
		let entry = verifier.external_entry(key).await.unwrap();
		entry.state.lock().await.last_cold_result =
			Some(Instant::now() - FAILED_REFRESH_COOLDOWN - Duration::from_secs(1));
	}
	let mut tasks = Vec::new();
	for key in &keys {
		let verifier = Arc::clone(&verifier);
		let key = key.clone();
		tasks.push(tokio::spawn(async move {
			let entry = verifier.external_entry(&key).await?;
			let mut state = entry.state.lock().await;
			verifier.refresh_external_entry(&mut state, &key).await
		}));
	}
	for _ in &keys {
		tokio::time::timeout(Duration::from_secs(5), started_rx.recv())
			.await
			.unwrap()
			.unwrap();
	}
	let excess = (namespace_id, "https://excess.example.com".to_owned());
	assert!(matches!(
		verifier.external_entry(&excess).await,
		Err(VerificationFailure::VerificationUnavailable)
	));
	let duplicate = {
		let verifier = Arc::clone(&verifier);
		let key = keys[0].clone();
		tokio::spawn(async move {
			let entry = verifier.external_entry(&key).await?;
			let mut state = entry.state.lock().await;
			verifier.refresh_external_entry(&mut state, &key).await
		})
	};
	assert_eq!(reads.load(Ordering::Relaxed), MAX_COLD_ISSUERS);
	pending.add_permits(MAX_COLD_ISSUERS);
	for task in tasks {
		assert!(matches!(
			task.await.unwrap(),
			Err(VerificationFailure::VerificationUnavailable)
		));
	}
	assert!(matches!(
		duplicate.await.unwrap(),
		Err(VerificationFailure::VerificationUnavailable)
	));
	assert_eq!(reads.load(Ordering::Relaxed), MAX_COLD_ISSUERS);
}

#[tokio::test]
async fn cancelled_cold_refreshes_release_the_cold_budget() {
	let pending = Arc::new(tokio::sync::Semaphore::new(0));
	let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
	let registry_fetcher: RegistryFetcher = Arc::new(move |_| {
		let pending = Arc::clone(&pending);
		let started_tx = started_tx.clone();
		Box::pin(async move {
			started_tx.send(()).unwrap();
			pending.acquire().await.unwrap().forget();
			bail!("registry unavailable")
		})
	});
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let namespace_id = Id::new_v1(1);
	let mut tasks = Vec::new();
	for index in 0..MAX_COLD_ISSUERS {
		let key = (namespace_id, format!("https://unknown-{index}.example.com"));
		verifier.external_cache.lock().await.insert(
			key.clone(),
			Arc::new(ExternalIssuerEntry {
				state: tokio::sync::Mutex::new(ExternalIssuerState::default()),
				first_seen: Instant::now() - FAILED_REFRESH_COOLDOWN - Duration::from_secs(1),
				verified: AtomicBool::new(false),
			}),
		);
		let verifier = Arc::clone(&verifier);
		tasks.push(tokio::spawn(async move {
			let entry = verifier.external_entry(&key).await?;
			let mut state = entry.state.lock().await;
			verifier.refresh_external_entry(&mut state, &key).await
		}));
	}
	for _ in 0..MAX_COLD_ISSUERS {
		tokio::time::timeout(Duration::from_secs(5), started_rx.recv())
			.await
			.unwrap()
			.unwrap();
	}
	for task in tasks {
		task.abort();
		assert!(task.await.unwrap_err().is_cancelled());
	}
	let new_key = (namespace_id, "https://new.example.com".to_owned());
	assert!(verifier.external_entry(&new_key).await.is_ok());
	assert_eq!(verifier.external_cache.lock().await.len(), MAX_COLD_ISSUERS);
}

#[tokio::test]
async fn failed_warm_refresh_reuses_stale_keys_without_retrying_each_waiter() {
	let reads = Arc::new(AtomicUsize::new(0));
	let registry_fetcher: RegistryFetcher = {
		let reads = Arc::clone(&reads);
		Arc::new(move |namespace_id| {
			let reads = Arc::clone(&reads);
			Box::pin(async move {
				if reads.fetch_add(1, Ordering::Relaxed) == 0 {
					Ok(ExternalIssuerRegistry::empty(namespace_id))
				} else {
					bail!("registry unavailable")
				}
			})
		})
	};
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let cache_key = (Id::new_v1(1), "https://issuer.example.com".to_owned());
	let entry = verifier.external_entry(&cache_key).await.unwrap();
	let mut state = entry.state.lock().await;
	verifier
		.refresh_external_entry(&mut state, &cache_key)
		.await
		.unwrap();
	state.cached.as_mut().unwrap().fetched_at = Instant::now() - Duration::from_secs(31);
	verifier
		.refresh_external_entry(&mut state, &cache_key)
		.await
		.unwrap();
	verifier
		.refresh_external_entry(&mut state, &cache_key)
		.await
		.unwrap();
	assert_eq!(reads.load(Ordering::Relaxed), 2);
}

#[tokio::test]
async fn unverified_hints_cannot_evict_registered_issuer() {
	let registry_fetcher: RegistryFetcher = Arc::new(|namespace_id| {
		Box::pin(async move { Ok(ExternalIssuerRegistry::empty(namespace_id)) })
	});
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let namespace_id = Id::new_v1(1);
	let registered_key = (namespace_id, "https://registered.example.com".to_owned());
	let registered = Arc::new(ExternalIssuerEntry::new());
	registered.verified.store(true, Ordering::Release);
	verifier
		.external_cache
		.lock()
		.await
		.insert(registered_key.clone(), Arc::clone(&registered));
	for index in 0..MAX_COLD_ISSUERS {
		let key = (namespace_id, format!("https://unknown-{index}.example.com"));
		verifier.external_entry(&key).await.unwrap();
	}
	let excess = (namespace_id, "https://excess.example.com".to_owned());
	assert!(matches!(
		verifier.external_entry(&excess).await,
		Err(VerificationFailure::VerificationUnavailable)
	));
	assert!(
		verifier
			.external_cache
			.lock()
			.await
			.get(&registered_key)
			.is_some_and(|entry| Arc::ptr_eq(entry, &registered))
	);
}

#[tokio::test]
async fn newly_verified_issuer_can_replace_a_full_positive_cache() {
	let registry_fetcher: RegistryFetcher = Arc::new(|namespace_id| {
		Box::pin(async move { Ok(ExternalIssuerRegistry::empty(namespace_id)) })
	});
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let namespace_id = Id::new_v1(1);
	for index in 0..MAX_CACHED_ISSUERS {
		let entry = Arc::new(ExternalIssuerEntry::new());
		entry.verified.store(true, Ordering::Release);
		verifier.external_cache.lock().await.insert(
			(
				namespace_id,
				format!("https://registered-{index}.example.com"),
			),
			entry,
		);
	}
	let new_key = (namespace_id, "https://new.example.com".to_owned());
	let new_entry = verifier.external_entry(&new_key).await.unwrap();
	assert_eq!(
		verifier.external_cache.lock().await.len(),
		MAX_CACHED_ISSUERS + 1
	);
	new_entry.verified.store(true, Ordering::Release);
	verifier.trim_verified_cache(&new_key, &new_entry).await;
	let caches = verifier.external_cache.lock().await;
	assert_eq!(caches.len(), MAX_CACHED_ISSUERS);
	assert!(
		caches
			.get(&new_key)
			.is_some_and(|entry| Arc::ptr_eq(entry, &new_entry))
	);
}

#[tokio::test]
async fn older_registry_never_restores_removed_or_reduced_issuer() {
	let namespace_id = Id::new_v1(1);
	let broad = vec![OperationKind::Read, OperationKind::Update];
	for narrowed in [None, Some(vec![OperationKind::Read])] {
		let sequence = Arc::new(StdMutex::new(VecDeque::from([
			registry(namespace_id, 1, Some(broad.clone())),
			registry(namespace_id, 2, narrowed.clone()),
			registry(namespace_id, 1, Some(broad.clone())),
		])));
		let registry_fetcher: RegistryFetcher = Arc::new(move |_| {
			let registry = sequence.lock().unwrap().pop_front().unwrap();
			Box::pin(async move { Ok(registry) })
		});
		let jwks_reads = Arc::new(AtomicUsize::new(0));
		let verifier = verifier_with_registry_source(registry_fetcher, jwks_reads.clone());
		let mut cached = None;
		let cache_key = (namespace_id, "https://issuer.example.com".into());
		verifier
			.refresh_external(&mut cached, &cache_key)
			.await
			.unwrap();
		verifier
			.refresh_external(&mut cached, &cache_key)
			.await
			.unwrap();
		let fetched_at = cached.as_ref().unwrap().fetched_at;
		let reads = jwks_reads.load(Ordering::Relaxed);
		verifier
			.refresh_external(&mut cached, &cache_key)
			.await
			.unwrap();
		let retained = cached.as_ref().unwrap();
		assert_eq!(retained.registry.generation, 2);
		assert_eq!(retained.fetched_at, fetched_at);
		assert_eq!(jwks_reads.load(Ordering::Relaxed), reads);
		assert_eq!(
			retained
				.registry
				.issuers
				.first()
				.map(|issuer| &issuer.grant_ceiling[0].operations),
			narrowed.as_ref(),
		);
	}
}

#[tokio::test]
async fn registry_read_has_time_to_outlast_a_blackholed_epoxy_replica() {
	let namespace_id = Id::new_v1(1);
	let registry_fetcher: RegistryFetcher = Arc::new(move |_| {
		Box::pin(async move {
			tokio::time::sleep(Duration::from_secs(6)).await;
			Ok(ExternalIssuerRegistry::empty(namespace_id))
		})
	});
	let verifier =
		verifier_with_registry_source(registry_fetcher, Arc::new(AtomicUsize::new(0)));
	let mut cached = None;
	verifier
		.refresh_external(
			&mut cached,
			&(namespace_id, "https://issuer.example.com".into()),
		)
		.await
		.unwrap();
	assert_eq!(cached.unwrap().registry.generation, 0);
}

#[test]
fn jwks_parser_requires_unique_public_keys_and_pinned_algorithm() {
	let x = URL_SAFE_NO_PAD.encode([7; 32]);
	let public = serde_json::json!({"kty":"OKP", "crv":"Ed25519", "kid":"key-1", "alg":"EdDSA", "use":"sig", "x":x});
	let encoded = |keys: Vec<serde_json::Value>| {
		serde_json::to_vec(&serde_json::json!({"keys": keys})).unwrap()
	};
	assert_eq!(
		parse_jwks(&encoded(vec![public.clone()]), "EdDSA")
			.unwrap()
			.len(),
		1
	);
	let with_metadata = serde_json::to_vec(&serde_json::json!({
		"keys": [{
			"kty":"OKP", "crv":"Ed25519", "kid":"key-1", "alg":"EdDSA",
			"use":"sig", "x":x, "vendor_metadata":"ignored"
		}],
		"issuer": "https://issuer.example.com"
	}))
	.unwrap();
	assert_eq!(parse_jwks(&with_metadata, "EdDSA").unwrap().len(), 1);
	assert!(parse_jwks(&encoded(vec![]), "EdDSA").unwrap().is_empty());
	let other_algorithms = serde_json::json!({"kty":"RSA", "kid":"rsa-key", "alg":"RS256"});
	let encryption_key = serde_json::json!({
		"kty":"OKP", "crv":"Ed25519", "kid":"encryption-key", "use":"enc", "x":x
	});
	assert_eq!(
		parse_jwks(
			&encoded(vec![public.clone(), other_algorithms, encryption_key]),
			"EdDSA"
		)
		.unwrap()
		.len(),
		1
	);
	assert!(
		parse_jwks(
			&encoded(vec![serde_json::json!({"kty":"RSA", "alg":"RS256"})]),
			"EdDSA"
		)
		.unwrap()
		.is_empty()
	);
	assert!(parse_jwks(&encoded(vec![public.clone(), public.clone()]), "EdDSA").is_err());
	assert!(
		parse_jwks(&encoded(vec![public.clone()]), "RS256")
			.unwrap()
			.is_empty()
	);
	for component in ["d", "p", "q", "dp", "dq", "qi", "oth", "k"] {
		let mut private = public.clone();
		private[component] = serde_json::json!(URL_SAFE_NO_PAD.encode([8; 32]));
		assert!(parse_jwks(&encoded(vec![private]), "EdDSA").is_err());
	}
	let mut wrong_use = public;
	wrong_use["use"] = serde_json::json!("enc");
	assert!(
		parse_jwks(&encoded(vec![wrong_use]), "EdDSA")
			.unwrap()
			.is_empty()
	);
}
#[test]
fn rejects_private_and_mapped_addresses() {
	for ip in [
		"127.0.0.1",
		"10.0.0.1",
		"100.100.100.100",
		"169.254.169.254",
		"172.16.0.1",
		"192.168.1.1",
		"192.0.2.1",
		"192.0.0.9",
		"192.88.99.1",
		"198.18.0.1",
		"::1",
		"fe80::1",
		"fc00::1",
		"2001:db8::1",
		"::ffff:127.0.0.1",
	] {
		assert!(!public_ip(ip.parse().unwrap()), "{ip} must be blocked");
	}
	assert!(public_ip("8.8.8.8".parse().unwrap()));
}

#[tokio::test]
async fn jwks_fetch_client_does_not_follow_redirects() {
	use tokio::io::{AsyncReadExt, AsyncWriteExt};

	let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
	let address = listener.local_addr().unwrap();
	let server = tokio::spawn(async move {
		let (mut connection, _) = listener.accept().await.unwrap();
		let mut request = [0; 1_024];
		connection.read(&mut request).await.unwrap();
		connection
			.write_all(b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/private\r\nContent-Length: 0\r\n\r\n")
			.await
			.unwrap();
	});
	let client = jwks_client("127.0.0.1", &[address]).unwrap();
	let response = client
		.get(format!("http://{address}/jwks.json"))
		.send()
		.await
		.unwrap();
	assert_eq!(response.status(), reqwest::StatusCode::FOUND);
	server.await.unwrap();
}
