use super::*;
use rivet_auth_policy::{OperationKind, ResourceKind, Scope};

#[test]
fn registry_round_trip_preserves_namespace_bound() {
	let namespace_id = Id::new_v1(1);
	let registry = ExternalIssuerRegistry {
		namespace_id,
		generation: 1,
		issuers: vec![ExternalIssuer {
			name: "example".into(),
			issuer: "https://issuer.example.com".into(),
			audience: "rivet-api".into(),
			jwks_url: "https://issuer.example.com/.well-known/jwks.json".into(),
			algorithm: "EdDSA".into(),
			grant_ceiling: vec![OwnedGrant {
				namespace: Scope::Id(namespace_id),
				resource: ResourceKind::Actor,
				target: Scope::Any,
				operations: vec![OperationKind::Read],
			}],
		}],
	};
	let decoded =
		ExternalIssuerRegistry::decode(namespace_id, &registry.encode().unwrap()).unwrap();
	assert_eq!(decoded.generation, 1);
	assert_eq!(decoded.issuers[0].grant_ceiling.len(), 1);
	assert!(
		ExternalIssuerRegistry::decode(Id::new_v1(2), &registry.encode().unwrap()).is_err()
	);
	for url in [
		"https://127.0.0.1/jwks.json",
		"https://[::1]/jwks.json",
		"https://issuer.example.com:8443/jwks.json",
		"https://user:secret@issuer.example.com/jwks.json",
		"https://issuer.example.com/jwks.json?key=secret",
		"http://issuer.example.com/jwks.json",
	] {
		let mut unsafe_registry = registry.clone();
		unsafe_registry.issuers[0].jwks_url = url.to_owned();
		assert!(unsafe_registry.validate().is_err(), "accepted {url}");
	}
}
