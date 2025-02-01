use chirp_workflow::prelude::*;
use nomad_client::apis::nodes_api;
use rivet_api::{
	apis::{
		configuration::Configuration,
		edge_intercom_pegboard_api::edge_intercom_pegboard_toggle_client_drain,
	},
	models,
};
use rivet_operation::prelude::proto::backend::pkg::*;

use crate::types::PoolType;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Input {
	pub datacenter_id: Uuid,
	pub server_id: Uuid,
	pub pool_type: PoolType,
}

#[workflow]
pub(crate) async fn cluster_server_drain(ctx: &mut WorkflowCtx, input: &Input) -> GlobalResult<()> {
	let drain_timeout = ctx
		.activity(GetDrainTimeoutInput {
			datacenter_id: input.datacenter_id,
			pool_type: input.pool_type,
		})
		.await?;

	match input.pool_type {
		PoolType::Job => {
			ctx.activity(DrainNodeInput {
				datacenter_id: input.datacenter_id,
				server_id: input.server_id,
				drain_timeout,
			})
			.await?;
		}
		PoolType::Gg => {
			ctx.signal(crate::workflows::server::DnsDelete {})
				.tag("server_id", input.server_id)
				.send()
				.await?;
		}
		PoolType::Pegboard | PoolType::PegboardIsolate => {
			ctx.activity(DrainPegboardAndDsInput {
				datacenter_id: input.datacenter_id,
				server_id: input.server_id,
				drain_timeout,
			})
			.await?;
		}
		PoolType::Ats | PoolType::Fdb => {}
	}

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub(crate) struct GetDrainTimeoutInput {
	pub datacenter_id: Uuid,
	pub pool_type: PoolType,
}

#[activity(GetDrainTimeout)]
pub(crate) async fn get_drain_timeout(
	ctx: &ActivityCtx,
	input: &GetDrainTimeoutInput,
) -> GlobalResult<u64> {
	let dcs_res = ctx
		.op(crate::ops::datacenter::get::Input {
			datacenter_ids: vec![input.datacenter_id],
		})
		.await?;
	let dc = unwrap!(dcs_res.datacenters.into_iter().next());

	let pool = unwrap!(
		dc.pools.iter().find(|p| p.pool_type == input.pool_type),
		"datacenter does not have this type of pool configured"
	);

	Ok(pool.drain_timeout)
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct DrainNodeInput {
	datacenter_id: Uuid,
	server_id: Uuid,
	drain_timeout: u64,
}

#[activity(DrainNode)]
async fn drain_node(ctx: &ActivityCtx, input: &DrainNodeInput) -> GlobalResult<()> {
	// Fetch drain ts from db to make the drain wf consistent if called more than once (in the event that
	// nomad/pb register after the drain started and this wf needs to be run again).
	let (drain_ts, nomad_node_id) = sql_fetch_one!(
		[ctx, (Option<i64>, Option<String>,)]
		"
		SELECT drain_ts, nomad_node_id
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		input.server_id,
	)
	.await?;

	let Some(nomad_node_id) = nomad_node_id else {
		return Ok(());
	};

	let nomad_config = nomad_util::new_build_config(ctx.config())?;
	let res = nodes_api::update_node_eligibility(
		&nomad_config,
		&nomad_node_id,
		nomad_client::models::NodeUpdateEligibilityRequest {
			eligibility: Some("ineligible".to_string()),
			node_id: Some(nomad_node_id.clone()),
		},
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
		None,
	)
	.await;

	// Catch "node not found" error
	if let Err(nomad_client::apis::Error::ResponseError(nomad_client::apis::ResponseContent {
		content,
		..
	})) = res
	{
		if content == "node not found" {
			tracing::warn!("node does not exist, not draining");
		}
	}

	// Prevent new matchmaker requests to the node running on this server
	msg!([ctx] mm::msg::nomad_node_closed_set(&nomad_node_id) {
		datacenter_id: Some(input.datacenter_id.into()),
		nomad_node_id: nomad_node_id.clone(),
		is_closed: true,
	})
	.await?;

	let drain_timeout = (drain_ts.unwrap_or_else(util::timestamp::now) as u64
		+ input.drain_timeout)
		.saturating_sub(util::timestamp::now() as u64);

	msg!([ctx] job_run::msg::drain_all(&nomad_node_id) {
		nomad_node_id: nomad_node_id.clone(),
		drain_timeout,
	})
	.await?;

	msg!([ctx] ds::msg::drain_all(&nomad_node_id) {
		nomad_node_id: Some(nomad_node_id.clone()),
		pegboard_client_id: None,
		drain_timeout,
	})
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct DrainPegboardAndDsInput {
	server_id: Uuid,
	datacenter_id: Uuid,
	drain_timeout: u64,
}

#[activity(DrainPegboardAndDs)]
async fn drain_pegboard_and_ds(
	ctx: &ActivityCtx,
	input: &DrainPegboardAndDsInput,
) -> GlobalResult<()> {
	// Fetch drain ts from db to make the drain wf consistent if called more than once (in the event that
	// nomad/pb register after the drain started and this wf needs to be run again).
	let (drain_ts, pegboard_client_id) = sql_fetch_one!(
		[ctx, (Option<i64>, Option<Uuid>)]
		"
		SELECT drain_ts, pegboard_client_id
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		input.server_id,
	)
	.await?;

	let Some(pegboard_client_id) = pegboard_client_id else {
		return Ok(());
	};

	let drain_complete_ts =
		drain_ts.unwrap_or_else(util::timestamp::now) + input.drain_timeout as i64;

	let config = Configuration {
		base_path: todo!("build url from dc id or name id"),
		bearer_access_token: todo!(),
		..Default::default()
	};

	edge_intercom_pegboard_toggle_client_drain(
		&config,
		&pegboard_client_id.to_string(),
		models::EdgeIntercomPegboardToggleClientDrainRequest {
			draining: true,
			drain_complete_ts: Some(util::timestamp::to_string(drain_complete_ts)?),
		},
	)
	.await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct DrainDynamicServersInput {
	pegboard_client_id: Uuid,
	drain_timeout: u64,
}

#[activity(DrainDynamicServers)]
async fn drain_dynamic_servers(
	ctx: &ActivityCtx,
	input: &DrainDynamicServersInput,
) -> GlobalResult<()> {
	msg!([ctx] ds::msg::drain_all(&input.pegboard_client_id) {
		nomad_node_id: None,
		pegboard_client_id: Some(input.pegboard_client_id.into()),
		drain_timeout: input.drain_timeout,
	})
	.await?;

	Ok(())
}
