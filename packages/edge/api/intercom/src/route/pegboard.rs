use api_helper::ctx::Ctx;
use chirp_workflow::prelude::*;
use cluster::types::BuildDeliveryMethod;
use fdb_util::SERIALIZABLE;
use foundationdb::{self as fdb, options::StreamingMode};
use futures_util::StreamExt;
use pegboard::protocol;
use rivet_api::models;
use serde_json::json;

use crate::auth::Auth;

// MARK: POST /pegboard/image/{}/prewarm
pub async fn prewarm_image(
	ctx: Ctx<Auth>,
	image_id: Uuid,
	body: serde_json::Value,
) -> GlobalResult<serde_json::Value> {
	ctx.auth().bypass()?;

	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;
	let (dc_res, servers_res, builds_res) = tokio::try_join!(
		ctx.op(cluster::ops::datacenter::get::Input {
			datacenter_ids: vec![dc_id],
		}),
		ctx.op(cluster::ops::server::list::Input {
			filter: cluster::types::Filter {
				datacenter_ids: Some(vec![dc_id]),
				pool_types: Some(vec![cluster::types::PoolType::Ats]),
				..Default::default()
			},
			include_destroyed: false,
			exclude_installing: true,
			exclude_draining: true,
			exclude_no_vlan: true,
		}),
		ctx.op(build::ops::get::Input {
			build_ids: vec![image_id],
		}),
	)?;

	let dc = unwrap!(dc_res.datacenters.first());
	let build = unwrap!(builds_res.builds.first());

	// Only prewarm if using ATS
	let BuildDeliveryMethod::TrafficServer = dc.build_delivery_method else {
		tracing::debug!("skipping prewarm since we're not using ats build delivery method");
		return Ok(json!({}));
	};

	if servers_res.servers.is_empty() {
		tracing::warn!(?dc_id, "no ats nodes to prewarm");
	}

	let artifact_url_stub = pegboard::util::image_artifact_url_stub(
		ctx.config(),
		build.upload_id,
		&build::utils::file_name(build.kind, build.compression),
	)?;
	let client = rivet_pools::reqwest::client().await?;

	futures_util::stream::iter(
		servers_res
			.servers
			.into_iter()
			.flat_map(|server| server.lan_ip.map(|lan_ip| (server, lan_ip))),
	)
	.map(|(server, lan_ip)| {
		let artifact_url_stub = artifact_url_stub.clone();
		let client = client.clone();

		async move {
			if let Err(err) = client
				.get(format!("http://{}:8080{}", lan_ip, &artifact_url_stub))
				.send()
				.await
				.and_then(|res| res.error_for_status())
			{
				tracing::error!(
					?err,
					server_id=?server.server_id,
					build_id=?build.build_id,
					"failed prewarming",
				);
			}
		}
	})
	.buffer_unordered(16)
	.collect::<()>()
	.await;

	Ok(json!({}))
}

// MARK: POST /pegboard/client/{}/toggle-drain
pub async fn toggle_drain_client(
	ctx: Ctx<Auth>,
	client_id: Uuid,
	body: models::EdgeIntercomPegboardToggleClientDrainRequest,
) -> GlobalResult<serde_json::Value> {
	ctx.auth().bypass()?;

	if body.draining {
		let res = ctx
			.signal(pegboard::workflows::client::Drain {
				drain_timeout_ts: unwrap_with!(
					body.drain_complete_ts,
					API_BAD_BODY,
					error = "missing `drain_complete_ts`"
				)
				.parse::<chrono::DateTime<chrono::Utc>>()?
				.timestamp_millis(),
			})
			.to_workflow::<pegboard::workflows::client::Workflow>()
			.tag("client_id", client_id)
			.send()
			.await;

		if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
			tracing::warn!(
				?client_id,
				"client workflow not found, likely already stopped"
			);
		} else {
			res?;
		}
	} else {
		let res = ctx
			.signal(pegboard::workflows::client::Undrain {})
			.to_workflow::<pegboard::workflows::client::Workflow>()
			.tag("client_id", client_id)
			.send()
			.await;

		if let Some(WorkflowError::WorkflowNotFound) = res.as_workflow_error() {
			tracing::warn!(
				?client_id,
				"client workflow not found, likely already stopped"
			);
		} else {
			res?;
		}
	}

	Ok(json!({}))
}
