use std::collections::HashSet;

use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend::pkg::*;

use crate::types::Filter;

pub struct Input {
	pub filter: Filter,
}

pub struct Output {}

#[operation]
pub async fn cluster_server_destroy_with_filter(
	ctx: &OperationCtx,
	input: &Input,
) -> GlobalResult<Output> {
	let servers_res = ctx
		.op(crate::ops::server::list::Input {
			filter: input.filter.clone(),
			include_destroyed: false,
		})
		.await?;

	// Flag as destroyed
	let server_ids = servers_res
		.servers
		.iter()
		.map(|x| x.server_id)
		.collect::<Vec<_>>();
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers
		SET cloud_destroy_ts = $2
		WHERE server_id = ANY($1)
		",
		&server_ids,
		util::timestamp::now(),
	)
	.await?;

	// Destroy server
	for server_id in server_ids {
		msg!([ctx] cluster::msg::server_destroy(server_id) {
			server_id: Some(server_id.into()),
			force: false,
		})
		.await?;
	}

	// Trigger scale event
	let dc_ids = servers_res
		.servers
		.iter()
		.map(|x| x.datacenter_id)
		.collect::<HashSet<_>>();
	for dc_id in dc_ids {
		msg!([ctx] cluster::msg::datacenter_scale(dc_id) {
			datacenter_id: Some(dc_id.into()),
		})
		.await?;
	}

	Ok(Output {})
}
