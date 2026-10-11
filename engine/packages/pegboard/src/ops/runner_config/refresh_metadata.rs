use anyhow::Result;
use gas::prelude::*;
use rivet_types::actor::RunnerPoolError;
use std::collections::HashMap;
use universaldb::prelude::*;

use crate::{
	keys,
	ops::serverless_metadata::fetch::{Output, ServerlessMetadataError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
	pub namespace_id: Id,
	pub runner_name: String,
	pub url: String,
	pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorNameEntry {
	pub name: String,
	pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[operation]
pub async fn pegboard_runner_config_refresh_metadata(
	ctx: &OperationCtx,
	input: &Input,
) -> Result<std::result::Result<Output, ServerlessMetadataError>> {
	let metadata = ctx
		.op(crate::ops::serverless_metadata::fetch::Input {
			url: input.url.clone(),
			headers: input.headers.clone(),
		})
		.await?;

	let metadata = match metadata {
		Ok(x) => x,
		Err(err) => return Ok(Err(err)),
	};

	// Save protocol to udb
	let downgraded = ctx
		.udb()?
		.txn("pegboard_runner_config_save_protocol", |tx| async move {
			let tx = tx.with_subspace(namespace::keys::subspace());

			let protocol_version_key = keys::runner_config::ProtocolVersionKey::new(
				input.namespace_id,
				input.runner_name.clone(),
			);

			if let Some(protocol_version) = metadata.envoy_protocol_version {
				tx.write(&protocol_version_key, protocol_version)?;

				Ok(false)
			} else if tx.exists(&protocol_version_key, Serializable).await? {
				Ok(true)
			} else {
				Ok(false)
			}
		})
		.await?;

	if downgraded {
		report_error(
			ctx,
			input.namespace_id,
			&input.runner_name,
			RunnerPoolError::Downgrade,
		)
		.await;
	}

	if metadata.envoy_protocol_version.is_some() {
		crate::utils::purge_runner_config_caches(
			ctx.cache(),
			input.namespace_id,
			&input.runner_name,
		)
		.await?;
	}

	// Full manifest, including an empty set, removes declarations from old deployments.
	ctx.udb()?
		.txn("pegboard_runner_config_write_actor_names", |tx| {
			let metadata = &metadata;
			async move {
				let tx = tx.with_subspace(keys::subspace());
				crate::actor_manifest::publish(
					&tx,
					input.namespace_id,
					&input.runner_name,
					&crate::actor_manifest::Manifest {
						supports_stream_subscriptions: metadata.stream_subscriptions,
						version: metadata
							.envoy_version
							.or(metadata.runner_version)
							.unwrap_or_default(),
						rivetkit_version: Some(metadata.version.clone()),
						actor_names: metadata
							.actor_names
							.iter()
							.map(|entry| (entry.name.clone(), entry.metadata.clone()))
							.collect(),
					},
				)
				.await
			}
		})
		.await?;

	Ok(Ok(metadata))
}

/// Report an error to the error tracker workflow.
async fn report_error(
	ctx: &OperationCtx,
	namespace_id: Id,
	pool_name: &str,
	error: RunnerPoolError,
) {
	if let Err(err) = ctx
		.signal(crate::workflows::runner_pool_error_tracker::ReportError { error })
		.bypass_signal_from_workflow_I_KNOW_WHAT_IM_DOING()
		.to_workflow::<crate::workflows::runner_pool_error_tracker::Workflow>()
		.tag("namespace_id", namespace_id)
		.tag("runner_name", pool_name)
		.graceful_not_found()
		.send()
		.await
	{
		tracing::warn!(?err, "failed to report serverless error");
	}
}
