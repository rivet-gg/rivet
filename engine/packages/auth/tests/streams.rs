use gas::prelude::Id;
use rivet_auth::{OperationKind, ResourceKind, TargetScope, streams::*};

#[test]
fn primary_stream_authority_is_bound_to_exact_storage_key_and_owner() {
	let owner = Id::new_v1(1);
	let path = format!("/durable-streams/v1/stream/actors/worker/{owner}");
	let key = storage_key("default", &path);
	assert_eq!(
		storage_target(&key, Some(&path), Some("default")).unwrap(),
		(ResourceKind::ActorStream, TargetScope::Id(owner))
	);
	assert!(storage_target(&key, Some(&path), Some("other")).is_err());
	assert!(storage_target(&key, Some("/other"), Some("default")).is_err());
	let generic = storage_target(&key, None, None).unwrap();
	assert_eq!(generic.0, ResourceKind::DurableStream);
	assert_ne!(generic.1, TargetScope::Id(owner));
}

#[test]
fn stream_operations_do_not_alias_read_delete_or_close() {
	assert_eq!(
		data_operation("POST", false, false).unwrap(),
		OperationKind::Append
	);
	assert_eq!(
		data_operation("GET", false, false).unwrap(),
		OperationKind::Read
	);
	assert_eq!(
		data_operation("DELETE", false, false).unwrap(),
		OperationKind::Delete
	);
	assert_eq!(
		data_operation("POST", true, false).unwrap(),
		OperationKind::Close
	);
}
