use super::*;
fn state() -> Data {
	Data {
		window_start: 10_000,
		application_operations: 0,
		webhook_operations: 0,
		webhook_bytes: 0,
		leader_datacenter: 1,
		leader_generation: 1,
	}
}
fn limits() -> rivet_config::config::auth::SigningQuota {
	rivet_config::config::auth::SigningQuota {
		application_operations: 2,
		webhook_operations: 1,
		webhook_bytes: 1024,
	}
}
#[test]
fn quota_restart_clock_rollback_reconfiguration_and_fencing() {
	let mut s = state();
	admit(&mut s, Charge::Application, limits(), 10_000, 1, 1).unwrap();
	let bytes = AuthSigningQuotaData::wrap_latest(s)
		.serialize_with_embedded_version(rivet_data::generated::AUTH_SIGNING_QUOTA_VERSION)
		.unwrap();
	let mut s = AuthSigningQuotaData::deserialize_with_embedded_version(&bytes).unwrap();
	assert_eq!(s.application_operations, 1);
	admit(&mut s, Charge::Application, limits(), 9_000, 1, 1).unwrap();
	assert!(admit(&mut s, Charge::Application, limits(), 9_000, 1, 1).is_err());
	let mut lower = limits();
	lower.application_operations = 1;
	assert!(admit(&mut s, Charge::Application, lower, 10_000, 1, 1).is_err());
	assert!(admit(&mut s, Charge::Webhook, limits(), 10_000, 2, 1).is_err());
	admit(&mut s, Charge::Webhook, limits(), 10_000, 2, 2).unwrap();
	assert!(admit(&mut s, Charge::Webhook, limits(), 10_000, 1, 1).is_err());
	assert!(admit(&mut s, Charge::Webhook, limits(), 10_000, 2, 2).is_err());
	admit(&mut s, Charge::Bytes(1000), limits(), 10_000, 2, 2).unwrap();
	assert!(admit(&mut s, Charge::Bytes(25), limits(), 10_000, 2, 2).is_err());
	admit(&mut s, Charge::Application, lower, 70_000, 2, 2).unwrap();
	assert_eq!(s.webhook_operations, 0);
	assert_eq!(s.webhook_bytes, 0);
}
