use api_helper::{
	anchor::{WatchIndexQuery, WatchResponse},
	ctx::Ctx,
};
use proto::backend;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::Deserialize;
use std::time::Duration;

use crate::{
	assert,
	auth::{Auth, CheckOpts, CheckOutput},
};

use super::GlobalQuery;

// MARK: GET /v2/actors/{}/logs
#[derive(Debug, Deserialize)]
pub struct GetActorLogsQuery {
	#[serde(flatten)]
	pub global: GlobalQuery,
	/// JSON-encoded user query expression for filtering logs
	pub query_json: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn get_logs(
	ctx: Ctx<Auth>,
	watch_index: WatchIndexQuery,
	query: GetActorLogsQuery,
) -> GlobalResult<models::ActorsGetActorLogsResponse> {
	let CheckOutput { game_id, env_id } = ctx
		.auth()
		.check(
			ctx.op_ctx(),
			CheckOpts {
				query: &query.global,
				allow_service_token: false,
				opt_auth: false,
			},
		)
		.await?;

	// Parse user query expression if provided
	let user_query_expr = if let Some(query_json) = &query.query_json {
		let expr = match serde_json::from_str::<clickhouse_user_query::QueryExpr>(query_json) {
			Ok(expr) => expr,
			Err(e) => {
				bail_with!(API_BAD_QUERY, error = e.to_string());
			}
		};
		Some(expr)
	} else {
		// No query provided, return empty result
		None
	};

	// Timestamp to start the query at
	let before_nts = util::timestamp::now() * 1_000_000;

	// Handle anchor
	let logs_res = if let Some(anchor) = watch_index.as_i64()? {
		let query_start = tokio::time::Instant::now();
		let user_query_expr_clone = user_query_expr.clone();

		// Poll for new logs
		let logs_res = loop {
			// Read logs after the timestamp
			//
			// We read descending in order to get at most 256 of the most recent logs. If we used
			// asc, we would be paginating through all the logs which would likely fall behind
			// actual stream and strain the database.
			//
			// We return fewer logs than the non-anchor request since this will be called
			// frequently and should not return a significant amount of logs.
			let logs_res = ctx
				.op(pegboard::ops::actor::log::read_with_query::Input {
					env_id,
					count: 64,
					order_by: pegboard::ops::actor::log::read_with_query::Order::Desc,
					query: pegboard::ops::actor::log::read_with_query::Query::AfterNts(anchor),
					user_query_expr: user_query_expr_clone.clone(),
				})
				.await?;

			// Return logs
			if !logs_res.entries.is_empty() {
				break logs_res;
			}

			// Timeout cleanly
			if query_start.elapsed().as_millis() > util::watch::DEFAULT_TIMEOUT as u128 {
				break pegboard::ops::actor::log::read_with_query::Output {
					entries: Vec::new(),
				};
			}

			// Throttle request
			//
			// We don't use `tokio::time::interval` because if the request takes longer than 500
			// ms, we'll enter a tight loop of requests.
			tokio::time::sleep(Duration::from_millis(1000)).await;
		};

		// Since we're using watch, we don't want this request to return immediately if there's new
		// results. Add an artificial timeout in order to prevent a tight loop if there's a high
		// log frequency.
		tokio::time::sleep_until(query_start + Duration::from_secs(1)).await;

		logs_res
	} else {
		// Read most recent logs
		ctx.op(pegboard::ops::actor::log::read_with_query::Input {
			env_id,
			count: 256,
			order_by: pegboard::ops::actor::log::read_with_query::Order::Desc,
			query: pegboard::ops::actor::log::read_with_query::Query::BeforeNts(before_nts),
			user_query_expr: user_query_expr.clone(),
		})
		.await?
	};

	// Convert to old Output format for compatibility
	let logs_res = pegboard::ops::actor::log::read::Output {
		entries: logs_res
			.entries
			.into_iter()
			.map(|e| pegboard::ops::actor::log::read::LogEntry {
				ts: e.ts,
				message: e.message,
				stream_type: e.stream_type,
				actor_id: e.actor_id,
			})
			.collect(),
	};

	// Build actor_ids map for lookup
	let mut actor_id_to_index: std::collections::HashMap<util::Id, i32> =
		std::collections::HashMap::new();
	let mut unique_actor_ids: Vec<String> = Vec::new();

	// Collect unique actor IDs and map them to indices
	for entry in &logs_res.entries {
		if !actor_id_to_index.contains_key(&entry.actor_id) {
			actor_id_to_index.insert(entry.actor_id.clone(), unique_actor_ids.len() as i32);
			unique_actor_ids.push(entry.actor_id.to_string());
		}
	}

	// Convert logs
	let mut lines = logs_res
		.entries
		.iter()
		.map(|entry| base64::encode(&entry.message))
		.collect::<Vec<_>>();
	let mut timestamps = logs_res
		.entries
		.iter()
		// Is nanoseconds
		.map(|x| x.ts / 1_000_000)
		.map(util::timestamp::to_string)
		.collect::<Result<Vec<_>, _>>()?;
	let mut streams = logs_res
		.entries
		.iter()
		.map(|x| x.stream_type as i32)
		.collect::<Vec<_>>();
	let mut foreigns = logs_res
		.entries
		.iter()
		.map(|x| x.foreign)
		.collect::<Vec<_>>();
	let mut actor_indices = logs_res
		.entries
		.iter()
		.map(|x| *actor_id_to_index.get(&x.actor_id).unwrap_or(&0))
		.collect::<Vec<_>>();

	// Order desc
	lines.reverse();
	timestamps.reverse();
	streams.reverse();
	foreigns.reverse();
	actor_indices.reverse();

	let watch_nts = logs_res.entries.first().map_or(before_nts, |x| x.ts);
	Ok(models::ActorsGetActorLogsResponse {
		actor_ids: unique_actor_ids,
		lines,
		timestamps,
		streams,
		foreigns,
		actor_indices,
		watch: WatchResponse::new_as_model(watch_nts),
	})
}
