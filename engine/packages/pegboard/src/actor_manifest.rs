//! Own stream declarations by pool and deployment, rather than last host to connect.
use anyhow::{Result, ensure};
use futures_util::TryStreamExt;
use gas::prelude::Id;
use rivet_data::converted::{ActorNameKeyData, MetadataKeyData};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use universaldb::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
	#[serde(default = "manifest_supports_subscriptions")]
	pub supports_stream_subscriptions: bool,
	pub version: u32,
	pub rivetkit_version: Option<String>,
	pub actor_names: BTreeMap<String, Map<String, Value>>,
}

fn manifest_supports_subscriptions() -> bool {
	true
}

/// Called in the caller's pegboard transaction, so the manifest and actor index
/// change together. Keeping empty source records makes removals win DC merges.
pub async fn publish(
	tx: &universaldb::Transaction,
	namespace: Id,
	pool: &str,
	manifest: &Manifest,
) -> Result<()> {
	if !manifest.supports_stream_subscriptions {
		// Older hosts prepopulate names, but cannot declare removals or replace
		// the Engine-owned index. Their payload may be incremental.
		for (name, metadata) in &manifest.actor_names {
			let key = crate::keys::ns::ActorNameKey::new(namespace, name.clone());
			let existing = tx.read_opt(&key, Serializable).await?;
			let metadata = legacy_metadata(existing.map(|data| data.metadata), metadata.clone());
			tx.write(&key, ActorNameKeyData { metadata })?;
		}
		return Ok(());
	}

	let key = crate::keys::actor_manifest::MetadataKey::new(namespace, pool.to_owned());
	let space = crate::keys::subspace().subspace(&key);
	let chunks = tx
		.get_ranges_keyvalues(universaldb::RangeOption::from(&space), Serializable)
		.try_collect::<Vec<_>>()
		.await?;
	let previous: Option<Manifest> = if chunks.is_empty() {
		None
	} else {
		Some(serde_json::from_value(Value::Object(
			key.combine(chunks)?.metadata,
		))?)
	};
	if !should_publish(previous.as_ref(), manifest)? {
		return Ok(());
	}
	let names: BTreeSet<_> = manifest
		.actor_names
		.keys()
		.chain(previous.iter().flat_map(|m| m.actor_names.keys()))
		.collect();
	for name in names {
		let actor_key = crate::keys::ns::ActorNameKey::new(namespace, name.clone());
		let mut data = tx
			.read_opt(&actor_key, Serializable)
			.await?
			.unwrap_or(ActorNameKeyData {
				metadata: Map::new(),
			});
		let mut sources = data
			.metadata
			.remove("streamSubscriptionSources")
			.and_then(|v| v.as_object().cloned())
			.unwrap_or_default();
		let declarations = manifest
			.actor_names
			.get(name)
			.and_then(|m| m.get("streamSubscriptions"))
			.cloned()
			.unwrap_or(json!({}));
		sources.insert(pool.to_owned(), json!({"version": manifest.version, "rivetkitVersion": manifest.rivetkit_version, "subscriptions": declarations}));
		if let Some(metadata) = manifest.actor_names.get(name) {
			data.metadata.extend(metadata.clone());
		}
		// This field is Engine-owned, never supplied by actor hosts.
		data.metadata
			.insert("streamSubscriptionSources".into(), Value::Object(sources));
		tx.write(&actor_key, data)?;
	}
	tx.delete_key_subspace(&key);
	let Value::Object(metadata) = serde_json::to_value(manifest)? else {
		unreachable!()
	};
	for (i, chunk) in key
		.split(MetadataKeyData { metadata })?
		.into_iter()
		.enumerate()
	{
		tx.set(&tx.pack(&key.chunk(i)), &chunk);
	}
	Ok(())
}

fn legacy_metadata(
	existing: Option<Map<String, Value>>,
	mut incoming: Map<String, Value>,
) -> Map<String, Value> {
	incoming.remove("streamSubscriptionSources");
	if let Some(sources) =
		existing.and_then(|mut metadata| metadata.remove("streamSubscriptionSources"))
	{
		incoming.insert("streamSubscriptionSources".into(), sources);
	}
	incoming
}

fn should_publish(previous: Option<&Manifest>, next: &Manifest) -> Result<bool> {
	let Some(previous) = previous else {
		return Ok(true);
	};
	if next.version < previous.version {
		return Ok(false);
	}
	if next.version == previous.version {
		ensure!(
			next == previous,
			"Actor metadata changed without incrementing the pool deployment version"
		);
		return Ok(false);
	}
	Ok(true)
}

/// Merge an independently replicated pool index without reviving removed rules.
/// Conflicting declarations at the same version fail closed (empty rules).
pub fn merge_sources(target: &mut Map<String, Value>, incoming: &Map<String, Value>) {
	let Some(incoming) = incoming
		.get("streamSubscriptionSources")
		.and_then(Value::as_object)
	else {
		return;
	};
	let sources = target
		.entry("streamSubscriptionSources")
		.or_insert_with(|| json!({}));
	let Some(sources) = sources.as_object_mut() else {
		return;
	};
	for (pool, next) in incoming {
		match sources.get_mut(pool) {
			Some(old) if old["version"].as_u64() > next["version"].as_u64() => {}
			Some(old) if old["version"] == next["version"] && old != next => {
				old["subscriptions"] = json!({});
				old["conflict"] = json!(true);
			}
			_ => {
				sources.insert(pool.clone(), next.clone());
			}
		}
	}
}

#[cfg(test)]
#[path = "../tests/unit/actor_manifest.rs"]
mod tests;
