use api_helper::{
	anchor::{WatchIndexQuery, WatchResponse},
	ctx::Ctx,
};
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::Deserialize;
use std::time::Duration;

use crate::{
	assert,
	auth::{Auth, CheckOpts, CheckOutput},
};

use super::GlobalQuery;

// MARK: GET /v1/container/{}/logs
#[derive(Debug, Deserialize)]
pub struct GetContainerLogsQuery {
	#[serde(flatten)]
	pub global: GlobalQuery,
	pub stream: models::ContainersQueryLogStream,
	pub container_ids_json: String,
	#[serde(default)]
	pub search_text: Option<String>,
	#[serde(default)]
	pub search_case_sensitive: Option<bool>,
	#[serde(default)]
	pub search_enable_regex: Option<bool>,
}

#[tracing::instrument(skip_all)]
pub async fn get_logs(
	ctx: Ctx<Auth>,
	watch_index: WatchIndexQuery,
	query: GetContainerLogsQuery,
) -> GlobalResult<models::ContainersGetContainerLogsResponse> {
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

	// Parse container IDs from the JSON string
	let container_ids: Vec<util::Id> = unwrap_with!(
		serde_json::from_str(&query.container_ids_json).ok(),
		CONTAINER_LOGS_INVALID_CONTAINER_IDS
	);

	ensure_with!(!container_ids.is_empty(), CONTAINER_LOGS_NO_CONTAINER_IDS);

	// Filter to only valid containers for this game/env
	let valid_container_ids =
		assert::actor_for_env(&ctx, &container_ids, game_id, env_id, None).await?;

	// Exit early if no valid containers
	ensure_with!(
		!valid_container_ids.is_empty(),
		CONTAINER_LOGS_NO_VALID_CONTAINER_IDS
	);

	// Use only the valid container IDs from now on
	let container_ids = valid_container_ids;

	// Determine stream type(s)
	let stream_types = match query.stream {
		models::ContainersQueryLogStream::StdOut => vec![pegboard::types::LogsStreamType::StdOut],
		models::ContainersQueryLogStream::StdErr => vec![pegboard::types::LogsStreamType::StdErr],
		models::ContainersQueryLogStream::All => vec![
			pegboard::types::LogsStreamType::StdOut,
			pegboard::types::LogsStreamType::StdErr,
		],
	};

	// Timestamp to start the query at
	let before_nts = util::timestamp::now() * 1_000_000;

	// Handle anchor
	let logs_res = if let Some(anchor) = watch_index.as_i64()? {
		let query_start = tokio::time::Instant::now();
		let stream_types_clone = stream_types.clone();
		let container_ids_clone = container_ids.clone();

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
				.op(pegboard::ops::actor::log::read::Input {
					actor_ids: container_ids_clone.clone(),
					stream_types: stream_types_clone.clone(),
					count: 64,
					order_by: pegboard::ops::actor::log::read::Order::Desc,
					query: pegboard::ops::actor::log::read::Query::AfterNts(anchor),
					search_text: query.search_text.clone(),
					search_case_sensitive: query.search_case_sensitive,
					search_enable_regex: query.search_enable_regex,
				})
				.await?;

			// Return logs
			if !logs_res.entries.is_empty() {
				break logs_res;
			}

			// Timeout cleanly
			if query_start.elapsed().as_millis() > util::watch::DEFAULT_TIMEOUT as u128 {
				break pegboard::ops::actor::log::read::Output {
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

		ctx.op(pegboard::ops::actor::log::read::Input {
			actor_ids: container_ids.clone(),
			stream_types: stream_types.clone(),
			count: 256,
			order_by: pegboard::ops::actor::log::read::Order::Desc,
			query: pegboard::ops::actor::log::read::Query::BeforeNts(before_nts),
			search_text: query.search_text.clone(),
			search_case_sensitive: query.search_case_sensitive,
			search_enable_regex: query.search_enable_regex,
		})
		.await?
	};

	// Build container_ids map for lookup
	let mut container_id_to_index: std::collections::HashMap<util::Id, i32> =
		std::collections::HashMap::new();
	let mut unique_container_ids: Vec<String> = Vec::new();

	// Collect unique container IDs and map them to indices
	for entry in &logs_res.entries {
		if !container_id_to_index.contains_key(&entry.actor_id) {
			container_id_to_index.insert(entry.actor_id.clone(), unique_container_ids.len() as i32);
			unique_container_ids.push(entry.actor_id.to_string());
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
	let mut container_indices = logs_res
		.entries
		.iter()
		.map(|x| *container_id_to_index.get(&x.actor_id).unwrap_or(&0))
		.collect::<Vec<_>>();

	// Order desc
	lines.reverse();
	timestamps.reverse();
	streams.reverse();
	foreigns.reverse();
	container_indices.reverse();

	let watch_nts = logs_res.entries.first().map_or(before_nts, |x| x.ts);
	Ok(models::ContainersGetContainerLogsResponse {
		container_ids: unique_container_ids,
		lines,
		timestamps,
		streams,
		foreigns,
		container_indices,
		watch: WatchResponse::new_as_model(watch_nts),
	})
}
