use super::*;

#[test]
fn management_admission_is_bounded_by_expiry_and_time_since_authentication_started() {
	assert_eq!(admission_deadline(10_000, None), 15_000);
	assert_eq!(admission_deadline(10_000, Some(12_000)), 12_000);
	assert_eq!(admission_deadline(10_000, Some(25_000)), 15_000);
	// The forwarded request retains the same deadline even if it arrives much later.
	let request = MutationRequest {
		namespace_id: Id::new_v1(1),
		change: Change::Rotate {
			expected_generation: 1,
		},
		admission_deadline_ts: Some(admission_deadline(10_000, Some(12_000))),
	};
	let wire = serde_json::to_vec(&request).unwrap();
	let received: MutationRequest = serde_json::from_slice(&wire).unwrap();
	assert!(
		rivet_auth_jwt::ops::webhook::mutate::check_admission(
			received.admission_deadline_ts,
			11_999
		)
		.is_ok()
	);
	assert!(
		rivet_auth_jwt::ops::webhook::mutate::check_admission(
			received.admission_deadline_ts,
			12_000
		)
		.is_err()
	);
}
