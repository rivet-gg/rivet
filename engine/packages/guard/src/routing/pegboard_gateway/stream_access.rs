use anyhow::{Result, ensure};
use base64::Engine as _;
use hyper::{HeaderMap, Method};
use rivet_auth::{OperationKind, ResourceKind, TargetScope, streams};

fn protocol_header(headers: &HeaderMap, name: &str) -> Result<Option<String>> {
	if let Some(value) = headers.get(format!("x-rivet-ds-h-{name}")) {
		let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(value.as_bytes())?;
		return Ok(Some(String::from_utf8(bytes)?));
	}
	headers
		.get(name)
		.map(|v| v.to_str().map(str::to_owned).map_err(Into::into))
		.transpose()
}

pub(super) fn storage_requests(
	key: &str,
	path: &str,
	method: &Method,
	headers: &HeaderMap,
) -> Result<Vec<(ResourceKind, TargetScope, OperationKind)>> {
	let header = |name| headers.get(name).and_then(|v| v.to_str().ok());
	let tenant = header(streams::TENANT_HEADER);
	let (resource, target) = streams::storage_target(key, header(streams::PATH_HEADER), tenant)?;
	if path != "/protocol" {
		return Ok(vec![(resource, target, OperationKind::Update)]);
	}
	let closed =
		protocol_header(headers, "stream-closed")?.is_some_and(|v| v.eq_ignore_ascii_case("true"));
	let parent = protocol_header(headers, "stream-forked-from")?;
	let operation = streams::data_operation(method.as_str(), closed, false)?;
	let mut requests = vec![(resource, target, operation)];
	if closed {
		requests.push((resource, target, OperationKind::Append));
	}
	if let Some(parent) = parent {
		ensure!(
			*method == Method::PUT && parent.starts_with('/'),
			"Invalid stream fork request"
		);
		let tenant = tenant.ok_or_else(|| anyhow::anyhow!("Forking requires stream identity"))?;
		let parent_key = streams::storage_key(tenant, &parent);
		let (resource, target) = streams::storage_target(&parent_key, Some(&parent), Some(tenant))?;
		requests.push((resource, target, OperationKind::Read));
	}
	Ok(requests)
}

#[cfg(test)]
#[path = "../../../tests/unit/stream_access.rs"]
mod tests;
