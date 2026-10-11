//! Narrow storage actor lookup/provisioning for stream-scoped credentials.
use crate::ctx::ApiCtx;
use anyhow::{Result, ensure};
use axum::http::HeaderMap;
use base64::Engine as _;
use rivet_auth::{AccessNamespaceScope, streams};

pub async fn authorize(
	ctx: &ApiCtx,
	namespace: &str,
	name: &str,
	key: &str,
	headers: &HeaderMap,
	input: Option<&str>,
) -> Result<bool> {
	if !headers.contains_key(streams::PATH_HEADER) {
		return Ok(false);
	}
	ensure!(
		name == streams::STORAGE_ACTOR_NAME,
		"Stream credentials require a storage actor"
	);
	let header = |name| headers.get(name).and_then(|v| v.to_str().ok());
	let path =
		header(streams::PATH_HEADER).ok_or_else(|| anyhow::anyhow!("Invalid stream path"))?;
	let tenant =
		header(streams::TENANT_HEADER).ok_or_else(|| anyhow::anyhow!("Missing stream tenant"))?;
	let (resource, target) = streams::storage_target(key, Some(path), Some(tenant))?;
	let operation = streams::lookup_operation(header(streams::OPERATION_HEADER).unwrap_or(""))?;
	ctx.auth(
		AccessNamespaceScope::Name(namespace.into()),
		resource,
		target,
		operation,
	)
	.await?;
	if let Some(input) = input {
		// Provisioning is only part of append authority; a read/delete grant
		// cannot allocate storage actors by calling the management API directly.
		ctx.auth(
			AccessNamespaceScope::Name(namespace.into()),
			resource,
			target,
			rivet_auth::OperationKind::Append,
		)
		.await?;
		let bytes = base64::engine::general_purpose::STANDARD.decode(input)?;
		let value: serde_json::Value = ciborium::from_reader(bytes.as_slice())?;
		ensure!(
			value["path"] == path && value["tenant_scope"] == tenant,
			"Stream input does not match the authorized storage identity"
		);
		ensure!(
			value.as_object().is_some_and(|v| v.len() == 2),
			"Unexpected stream input fields"
		);
	}
	Ok(true)
}
