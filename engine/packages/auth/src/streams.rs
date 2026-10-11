//! Stream authority is bound to the storage actor's deterministic key, never to
//! a caller-supplied resource selector alone. Primary streams use their owner's
//! actor ID; other streams use the first 128 bits of the SHA-256 storage key.
use crate::{OperationKind, ResourceKind, TargetScope};
use anyhow::{Result, ensure};
use gas::prelude::Id;
use sha2::{Digest, Sha256};

pub const STORAGE_ACTOR_NAME: &str = "durableStream";
pub const PATH_HEADER: &str = "x-rivet-stream-path";
pub const TENANT_HEADER: &str = "x-rivet-stream-tenant";
pub const OPERATION_HEADER: &str = "x-rivet-stream-operation";

pub fn storage_key(tenant: &str, path: &str) -> String {
	let mut hash = Sha256::new();
	hash.update(tenant.as_bytes());
	hash.update([0]);
	hash.update(path.as_bytes());
	format!("{:x}", hash.finalize())
}

pub fn storage_target(
	key: &str,
	path: Option<&str>,
	tenant: Option<&str>,
) -> Result<(ResourceKind, TargetScope)> {
	ensure!(
		key.len() == 64
			&& key
				.bytes()
				.all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
		"Invalid stream actor key"
	);
	if let Some(path) = path {
		let tenant = tenant.ok_or_else(|| anyhow::anyhow!("Missing stream tenant"))?;
		ensure!(
			path.starts_with('/')
				&& !path.contains(['?', '#', '\\'])
				&& storage_key(tenant, path) == key,
			"Stream path does not match actor key"
		);
		if let Some((_, suffix)) = path.rsplit_once("/actors/") {
			let parts: Vec<_> = suffix.split('/').collect();
			if parts.len() == 2 && !parts[0].is_empty() {
				if let Ok(id) = Id::parse(parts[1]) {
					return Ok((ResourceKind::ActorStream, TargetScope::Id(id)));
				}
			}
		}
	} else {
		ensure!(tenant.is_none(), "Stream tenant without path");
	}
	let mut bytes = [0u8; 18];
	for (i, byte) in bytes[..16].iter_mut().enumerate() {
		*byte = u8::from_str_radix(&key[i * 2..i * 2 + 2], 16)?;
	}
	Ok((ResourceKind::DurableStream, TargetScope::Id(Id::V1(bytes))))
}

/// These operations are distinct even when transported by the same HTTP method.
pub fn data_operation(method: &str, closed: bool, fork: bool) -> Result<OperationKind> {
	ensure!(!fork, "Forking requires separate source authorization");
	Ok(match (method, closed) {
		("GET" | "HEAD", false) => OperationKind::Read,
		("POST" | "PUT", true) => OperationKind::Close,
		("POST" | "PUT", false) => OperationKind::Append,
		("DELETE", false) => OperationKind::Delete,
		_ => anyhow::bail!("Invalid stream operation"),
	})
}

pub fn lookup_operation(value: &str) -> Result<OperationKind> {
	Ok(match value {
		"append" => OperationKind::Append,
		"read" => OperationKind::Read,
		"close" => OperationKind::Close,
		"delete" => OperationKind::Delete,
		_ => anyhow::bail!("Invalid stream lookup operation"),
	})
}
