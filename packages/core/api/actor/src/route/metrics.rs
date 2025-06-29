use api_helper::{
	anchor::{WatchIndexQuery, WatchResponse},
	ctx::Ctx,
};
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use crate::{
	assert,
	auth::{Auth, CheckOpts, CheckOutput},
	utils::build_global_query_compat,
};

use super::GlobalQuery;

// MARK: GET /actors/metrics/history
#[derive(Debug, Deserialize)]
pub struct GetActorMetricsQuery {
	#[serde(flatten)]
	pub global: GlobalQuery,
	pub start: i64,
	pub end: i64,
	pub interval: i64,
	pub actor_ids_json: String,
	pub metrics_json: String,
}

#[derive(Debug, clickhouse::Row, serde::Deserialize)]
pub struct MetricRow {
	pub time_bucket: i64,
	pub actor_id_str: String,
	pub metric_name: String,
	pub value: f64,
}

#[tracing::instrument(skip_all)]
pub async fn get_metrics(
	ctx: Ctx<Auth>,
	watch_index: WatchIndexQuery,
	query: GetActorMetricsQuery,
) -> GlobalResult<models::ActorsGetActorMetricsResponse> {
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

	// Parse metrics from the JSON string
	let requested_metrics: Vec<String> = unwrap_with!(
		serde_json::from_str(&query.metrics_json).ok(),
		ACTOR_METRICS_INVALID_METRICS
	);

	ensure_with!(!requested_metrics.is_empty(), ACTOR_METRICS_NO_METRICS);

	// Filter to only valid actors for this game/env
	let valid_actor_ids = assert::actor_for_env(&ctx, &actor_ids, game_id, env_id, None).await?;

	// Exit early if no valid actors
	ensure_with!(!valid_actor_ids.is_empty(), ACTOR_LOGS_NO_VALID_ACTOR_IDS);

	// Use only the valid actor IDs from now on
	let actor_ids = valid_actor_ids;

	// Get ClickHouse client
	let clickhouse = ctx.clickhouse().await?;

	// Convert actor IDs to strings for the query
	let actor_id_strings: Vec<String> = actor_ids.iter().map(|id| id.to_string()).collect();

	// Map common metric names to ClickHouse metric names
	let metric_mapping = get_metric_mapping();
	let clickhouse_metrics: Vec<String> = requested_metrics
		.iter()
		.filter_map(|m| metric_mapping.get(m).cloned())
		.collect();

	ensure_with!(
		!clickhouse_metrics.is_empty(),
		ACTOR_METRICS_UNSUPPORTED_METRICS
	);

	// Convert milliseconds to seconds for ClickHouse
	let start_seconds = query.start / 1000;
	let end_seconds = query.end / 1000;
	let interval_seconds = query.interval / 1000;

	// Build the query - try to use container label first, fallback to container name parsing
	let query_sql = formatdoc!(
		"
		SELECT
			toUnixTimestamp(toStartOfInterval(TimeUnix, INTERVAL ? SECOND)) as time_bucket,
			COALESCE(
				ResourceAttributes['container_label_com_rivet_actor_id'],
				if(
					match(Attributes['container_name'], '^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}-[0-9]+$'),
					substring(Attributes['container_name'], 1, 36),
					''
				)
			) as actor_id_str,
			MetricName as metric_name,
			avg(Value) as value
		FROM otel.otel_metrics_gauge
		WHERE
			TimeUnix >= fromUnixTimestamp(?)
			AND TimeUnix <= fromUnixTimestamp(?)
			AND (
				ResourceAttributes['container_label_com_rivet_actor_id'] IN ?
				OR substring(Attributes['container_name'], 1, 36) IN ?
			)
			AND MetricName IN ?
			AND actor_id_str != ''
		GROUP BY time_bucket, actor_id_str, metric_name
		ORDER BY time_bucket ASC, actor_id_str, metric_name
		"
	);

	let rows = clickhouse
		.query(&query_sql)
		.bind(interval_seconds)
		.bind(start_seconds)
		.bind(end_seconds)
		.bind(&actor_id_strings)
		.bind(&actor_id_strings) // Used twice in the query
		.bind(&clickhouse_metrics)
		.fetch_all::<MetricRow>()
		.await
		.map_err(|err| GlobalError::from(err))?;

	// Process the results
	let mut actor_metrics: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
	let mut time_buckets: Vec<i64> = Vec::new();

	// Initialize data structures
	for actor_id in &actor_id_strings {
		actor_metrics.insert(actor_id.clone(), HashMap::new());
		for metric in &requested_metrics {
			actor_metrics
				.get_mut(actor_id)
				.unwrap()
				.insert(metric.clone(), Vec::new());
		}
	}

	// Generate time buckets in seconds (since ClickHouse returns seconds)
	let mut current_time = start_seconds;
	while current_time <= end_seconds {
		time_buckets.push(current_time);
		current_time += interval_seconds;
	}

	// Fill in the data
	let reverse_metric_mapping: HashMap<String, String> = metric_mapping
		.iter()
		.map(|(k, v)| (v.clone(), k.clone()))
		.collect();

	for row in rows {
		if let (Some(original_metric), Some(actor_metrics_map)) = (
			reverse_metric_mapping.get(&row.metric_name),
			actor_metrics.get_mut(&row.actor_id_str),
		) {
			if let Some(metric_values) = actor_metrics_map.get_mut(original_metric) {
				// Find the index for this time bucket
				if let Some(bucket_index) = time_buckets.iter().position(|&t| t == row.time_bucket)
				{
					// Extend the vector if needed
					while metric_values.len() <= bucket_index {
						metric_values.push(0.0);
					}
					metric_values[bucket_index] = row.value;
				}
			}
		}
	}

	// Fill in missing values with 0.0 and ensure all vectors are the same length
	for actor_id in &actor_id_strings {
		if let Some(actor_metrics_map) = actor_metrics.get_mut(actor_id) {
			for metric_values in actor_metrics_map.values_mut() {
				while metric_values.len() < time_buckets.len() {
					metric_values.push(0.0);
				}
			}
		}
	}

	// Prepare the response format: metrics[metric_index][time_index] = value
	// The API expects metrics: Vec<Vec<f64>> where outer vec is per metric, inner vec is per time
	let mut response_metrics: Vec<Vec<f64>> = Vec::new();

	for metric in &requested_metrics {
		let mut metric_time_series: Vec<f64> = Vec::new();

		for time_bucket in &time_buckets {
			let mut total_value = 0.0;
			let mut count = 0;

			// Aggregate across all actors for this metric at this time
			for actor_id in &actor_id_strings {
				if let Some(actor_metrics_map) = actor_metrics.get(actor_id) {
					if let Some(metric_values) = actor_metrics_map.get(metric) {
						if let Some(bucket_index) =
							time_buckets.iter().position(|&t| t == *time_bucket)
						{
							if bucket_index < metric_values.len() {
								total_value += metric_values[bucket_index];
								count += 1;
							}
						}
					}
				}
			}

			// Use average value across actors
			metric_time_series.push(if count > 0 {
				total_value / count as f64
			} else {
				0.0
			});
		}

		response_metrics.push(metric_time_series);
	}

	Ok(models::ActorsGetActorMetricsResponse {
		actor_ids: actor_id_strings,
		metrics: response_metrics,
	})
}

fn get_metric_mapping() -> HashMap<String, String> {
	let mut mapping = HashMap::new();

	// Map user-friendly metric names to ClickHouse metric names from cAdvisor
	mapping.insert(
		"cpu".to_string(),
		"container_cpu_usage_seconds_total".to_string(),
	);
	mapping.insert(
		"memory".to_string(),
		"container_memory_usage_bytes".to_string(),
	);
	mapping.insert(
		"memory_limit".to_string(),
		"container_spec_memory_limit_bytes".to_string(),
	);
	mapping.insert(
		"network_rx_bytes".to_string(),
		"container_network_receive_bytes_total".to_string(),
	);
	mapping.insert(
		"network_tx_bytes".to_string(),
		"container_network_transmit_bytes_total".to_string(),
	);

	mapping
}
