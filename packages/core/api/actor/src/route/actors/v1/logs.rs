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
	utils::build_global_query_compat,
};

use super::GlobalQuery;

// MARK: GET /actors/{}/logs
#[derive(Debug, Deserialize)]
pub struct GetActorLogsQuery {
	#[serde(flatten)]
	pub global: GlobalQuery,
	pub stream: models::ActorsV1QueryLogStream,
	pub actor_ids_json: String,
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
	query: GetActorLogsQuery,
) -> GlobalResult<models::ActorsV1GetActorLogsResponse> {
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

	// Parse actor IDs from the JSON string
	let actor_ids: Vec<Uuid> = unwrap_with!(
		serde_json::from_str(&query.actor_ids_json).ok(),
		ACTOR_LOGS_INVALID_ACTOR_IDS
	);

	ensure_with!(!actor_ids.is_empty(), ACTOR_LOGS_NO_ACTOR_IDS);

	// Filter to only valid actors for this game/env
	let valid_actor_ids = assert::actor_for_env_v1(&ctx, &actor_ids, game_id, env_id, None).await?;

	// Exit early if no valid actors
	ensure_with!(!valid_actor_ids.is_empty(), ACTOR_LOGS_NO_VALID_ACTOR_IDS);

	// Use only the valid actor IDs from now on
	let actor_ids = valid_actor_ids;

	// Determine stream type(s)
	let stream_types = match query.stream {
		models::ActorsV1QueryLogStream::StdOut => vec![pegboard::types::LogsStreamType::StdOut],
		models::ActorsV1QueryLogStream::StdErr => vec![pegboard::types::LogsStreamType::StdErr],
		models::ActorsV1QueryLogStream::All => vec![
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
		let actor_ids_clone = actor_ids.clone();

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
				.op(pegboard::ops::actor::v1::log::read::Input {
					actor_ids: actor_ids_clone.clone(),
					stream_types: stream_types_clone.clone(),
					count: 64,
					order_by: pegboard::ops::actor::v1::log::read::Order::Desc,
					query: pegboard::ops::actor::v1::log::read::Query::AfterNts(anchor),
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
				break pegboard::ops::actor::v1::log::read::Output {
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

		ctx.op(pegboard::ops::actor::v1::log::read::Input {
			actor_ids: actor_ids.clone(),
			stream_types: stream_types.clone(),
			count: 256,
			order_by: pegboard::ops::actor::v1::log::read::Order::Desc,
			query: pegboard::ops::actor::v1::log::read::Query::BeforeNts(before_nts),
			search_text: query.search_text.clone(),
			search_case_sensitive: query.search_case_sensitive,
			search_enable_regex: query.search_enable_regex,
		})
		.await?
	};

	// Build actor_ids map for lookup
	let mut actor_id_to_index: std::collections::HashMap<Uuid, i32> =
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
	let mut actor_indices = logs_res
		.entries
		.iter()
		.map(|x| *actor_id_to_index.get(&x.actor_id).unwrap_or(&0))
		.collect::<Vec<_>>();

	// Order desc
	lines.reverse();
	timestamps.reverse();
	streams.reverse();
	actor_indices.reverse();

	let watch_nts = logs_res.entries.first().map_or(before_nts, |x| x.ts);
	Ok(models::ActorsV1GetActorLogsResponse {
		actor_ids: unique_actor_ids,
		lines,
		timestamps,
		streams,
		actor_indices,
		watch: WatchResponse::new_as_model(watch_nts),
	})
}

pub async fn get_logs_deprecated(
	ctx: Ctx<Auth>,
	game_id: Uuid,
	env_id: Uuid,
	server_id: Uuid,
	watch_index: WatchIndexQuery,
	query: GetActorLogsQuery,
) -> GlobalResult<models::ServersGetServerLogsResponse> {
	let global = build_global_query_compat(&ctx, game_id, env_id).await?;

	// Build a proper query expression for the single actor
	let query_expr = clickhouse_user_query::QueryExpr::StringEqual {
		property: "actor_id".to_string(),
		map_key: None,
		value: server_id.to_string(),
		case_insensitive: false,
	};
	let query_json = Some(serde_json::to_string(&query_expr)?);

	let logs_res = get_logs(ctx, watch_index, GetActorLogsQuery { global, query_json }).await?;
	Ok(models::ServersGetServerLogsResponse {
		lines: logs_res.lines,
		timestamps: logs_res.timestamps,
		// streams are not part of the deprecated response
		watch: logs_res.watch,
	})
}

// MARK: POST /actors/logs/export
#[tracing::instrument(skip_all)]
pub async fn export_logs(
	ctx: Ctx<Auth>,
	body: models::ActorsLogsExportRequest,
) -> GlobalResult<models::ActorsExportActorLogsResponse> {
	let CheckOutput { game_id, env_id } = ctx
		.auth()
		.check(
			ctx.op_ctx(),
			CheckOpts {
				query: &GlobalQuery {
					project: body.project,
					environment: body.environment,
				},
				allow_service_token: false,
				opt_auth: false,
			},
		)
		.await?;

	// Parse user query expression if provided
	let user_query_expr = if let Some(query_json) = &body.query_json {
		let expr = match serde_json::from_str::<clickhouse_user_query::QueryExpr>(query_json) {
			Ok(expr) => expr,
			Err(e) => {
				bail_with!(API_BAD_QUERY, error = e.to_string());
			}
		};
		Some(expr)
	} else {
		// No query provided, return empty result
		return Ok(models::ActorsExportActorLogsResponse { url: String::new() });
	};

	// Read all logs (no limit)
	let logs_res = ctx
		.op(pegboard::ops::actor::log::read_with_query::Input {
			env_id,
			count: i64::MAX,
			order_by: pegboard::ops::actor::log::read_with_query::Order::Asc,
			query: pegboard::ops::actor::log::read_with_query::Query::All,
			user_query_expr,
		})
		.await?;

	// Format logs as plain text
	let mut output = String::new();
	for entry in &logs_res.entries {
		let timestamp = util::timestamp::to_string(entry.ts / 1_000_000)?;
		let stream_name = match entry.stream_type {
			x if x == (pegboard::types::LogsStreamType::StdOut as u8) => "stdout",
			x if x == (pegboard::types::LogsStreamType::StdErr as u8) => "stderr",
			x => bail!("unknown stream type {x}"),
		};
		let message = String::from_utf8_lossy(&entry.message);
		output.push_str(&format!("{} [{}] {}\n", timestamp, stream_name, message));
	}

	// Generate filename
	let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
	let filename = format!("actor_logs_export_{}.txt", timestamp);

	// Upload to S3
	let mime = "text/plain";
	let content_length = output.len();
	let upload_res = op!([ctx] upload_prepare {
		bucket: "bucket-actor-log-export".into(),
		files: vec![
			backend::upload::PrepareFile {
				path: filename.clone(),
				mime: Some(mime.into()),
				content_length: content_length as u64,
				..Default::default()
			},
		],
		presigned_endpoint_kind: Some(backend::pkg::upload::prepare::EndpointKind::Internal as i32),
	})
	.await?;
	let upload_id = unwrap!(upload_res.upload_id).as_uuid();

	let presigned_req = unwrap!(upload_res.presigned_requests.first());

	// Upload the content
	let res = reqwest::Client::new()
		.put(&presigned_req.url)
		.body(output)
		.header(reqwest::header::CONTENT_TYPE, mime)
		.header(reqwest::header::CONTENT_LENGTH, content_length)
		.send()
		.await?;

	if !res.status().is_success() {
		let status = res.status();
		let text = res.text().await;
		tracing::error!(?status, ?text, "failed to upload logs");
		bail!("failed to upload logs");
	}

	op!([ctx] upload_complete {
		upload_id: upload_res.upload_id,
		bucket: Some("bucket-actor-log-export".into()),
	})
	.await?;

	// Generate download URL (valid for 1 hour)
	let s3_client = s3_util::Client::with_bucket_and_endpoint(
		ctx.config(),
		"bucket-actor-log-export",
		s3_util::EndpointKind::External,
	)
	.await?;
	let presigned_req = s3_client
		.get_object()
		.bucket(s3_client.bucket())
		.key(format!("{upload_id}/{filename}"))
		.presigned(
			s3_util::aws_sdk_s3::presigning::PresigningConfig::builder()
				.expires_in(Duration::from_secs(60 * 60))
				.build()?,
		)
		.await?;

	Ok(models::ActorsExportActorLogsResponse {
		url: presigned_req.uri().to_string(),
	})
}
