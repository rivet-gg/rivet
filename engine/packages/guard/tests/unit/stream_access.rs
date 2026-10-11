use super::*;

#[test]
fn append_read_delete_and_bridged_close_are_distinct() {
	let path = "/durable-streams/v1/stream/events";
	let key = streams::storage_key("default", path);
	let mut headers = HeaderMap::new();
	headers.insert(streams::PATH_HEADER, path.parse().unwrap());
	headers.insert(streams::TENANT_HEADER, "default".parse().unwrap());
	for (method, operation) in [
		(Method::POST, OperationKind::Append),
		(Method::GET, OperationKind::Read),
		(Method::DELETE, OperationKind::Delete),
	] {
		assert_eq!(
			storage_requests(&key, "/protocol", &method, &headers).unwrap()[0].2,
			operation
		);
	}
	headers.insert(
		"x-rivet-ds-h-stream-closed",
		base64::engine::general_purpose::URL_SAFE_NO_PAD
			.encode("TRUE")
			.parse()
			.unwrap(),
	);
	let requests = storage_requests(&key, "/protocol", &Method::POST, &headers).unwrap();
	assert_eq!(requests[0].2, OperationKind::Close);
	assert_eq!(requests[1].2, OperationKind::Append);
	headers.insert(streams::PATH_HEADER, "/other".parse().unwrap());
	assert!(storage_requests(&key, "/protocol", &Method::POST, &headers).is_err());
}

#[test]
fn fork_requires_source_read_and_storage_rpc_is_not_append() {
	let path = "/events";
	let key = streams::storage_key("default", path);
	let mut headers = HeaderMap::new();
	headers.insert(streams::PATH_HEADER, path.parse().unwrap());
	headers.insert(streams::TENANT_HEADER, "default".parse().unwrap());
	headers.insert("stream-forked-from", "/source".parse().unwrap());
	let requests = storage_requests(&key, "/protocol", &Method::PUT, &headers).unwrap();
	assert_eq!(requests[0].2, OperationKind::Append);
	assert_eq!(requests[1].2, OperationKind::Read);
	assert_ne!(requests[0].1, requests[1].1);
	assert_eq!(
		storage_requests(&key, "/action/internal", &Method::POST, &headers).unwrap()[0].2,
		OperationKind::Update
	);
}
