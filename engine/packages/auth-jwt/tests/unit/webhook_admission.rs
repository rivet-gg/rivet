use super::mutate::check_admission;

#[test]
fn webhook_management_admission_expires_before_a_late_proposal_or_retry() {
	assert!(check_admission(Some(12_000), 11_999).is_ok());
	assert!(check_admission(Some(12_000), 12_000).is_err());
	assert!(check_admission(Some(12_000), 15_000).is_err());
	// Lifecycle reconciliation is independent of a caller credential.
	assert!(check_admission(None, i64::MAX).is_ok());
}
