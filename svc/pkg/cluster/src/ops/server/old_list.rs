use std::{collections::HashSet, convert::TryFrom};

use chirp_workflow::prelude::*;
use linode::util::client;
use reqwest::header;
use serde_json::json;

use crate::types::Provider;

#[derive(Deserialize)]
struct GetLinodesResponse {
	data: Vec<Linode>,
}

#[derive(Deserialize)]
struct Linode {
	created: chrono::NaiveDateTime,
	label: String,
}

#[derive(Debug)]
pub struct Input {
	pub cluster_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
	pub server_ids: Vec<Uuid>,
}

/// Fetches servers directly from the cloud providers own APIs and returns servers older than 12 hours.
#[operation]
pub async fn old_list(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let linode_token = util::env::read_secret(&["linode", "token"]).await?;

	let dc_rows = sql_fetch_all!(
		[ctx, (i64, Option<String>)]
		"
		SELECT provider, provider_api_token
		FROM db_cluster.datacenters
		WHERE
			provider_api_token IS NOT NULL AND
			cluster_id = ANY($1)
		",
		&input.cluster_ids,
	)
	.await?;

	let accounts = dc_rows
		.iter()
		.map(|(provider, provider_api_token)| {
			let provider = Provider::try_from(*provider)?;
			// Default token if none is set
			let provider_api_token = match &provider {
				Provider::Linode => provider_api_token
					.clone()
					.unwrap_or_else(|| linode_token.clone()),
			};

			Ok((provider, provider_api_token))
		})
		.collect::<GlobalResult<HashSet<_>>>()?;

	// Filter by namespace
	let filter = json!({
		"label": {
			"+contains": format!("{}-", util::env::namespace()),
		}
	});
	let mut headers = header::HeaderMap::new();
	headers.insert(
		"X-Filter",
		header::HeaderValue::from_str(&serde_json::to_string(&filter)?)?,
	);

	let mut server_ids = Vec::new();
	for (provider, api_token) in accounts {
		match provider {
			Provider::Linode => {
				server_ids.extend(
					run_for_linode_account(ctx, &input.cluster_ids, &api_token, &headers).await?,
				);
			}
		}
	}

	Ok(Output { server_ids })
}

async fn run_for_linode_account(
	ctx: &OperationCtx,
	cluster_ids: &[Uuid],
	api_token: &str,
	headers: &header::HeaderMap,
) -> GlobalResult<Vec<Uuid>> {
	// Build HTTP client
	let client =
		client::Client::new_with_headers(Some(api_token.to_string()), headers.clone()).await?;

	let req = client
		.inner()
		.get("https://api.linode.com/v4/linode/instances")
		.query(&[("page_size", 500)]);

	let res = client
		.request(req, None, false)
		.await?
		.json::<GetLinodesResponse>()
		.await?;

	tracing::info!("{} servers in account", res.data.len());

	let server_ids = res
		.data
		.into_iter()
		// Filter out servers younger than 12 hours
		.filter(|linode| {
			linode.created.and_utc().timestamp_millis() < ctx.ts() - util::duration::hours(12)
		})
		.map(|linode| {
			util::uuid::parse(unwrap!(linode
				.label
				.get(util::env::namespace().len() + 1..)))
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	// Select deleted servers that match the linode api call
	let server_ids = sql_fetch_all!(
		[ctx, (Uuid,)]
		"
		SELECT server_id
		FROM db_cluster.servers AS s
		JOIN db_cluster.datacenters AS d
		ON s.datacenter_id = d.datacenter_id
		WHERE
			server_id = ANY($1) AND
			cluster_id = ANY($1) AND
			cloud_destroy_ts IS NOT NULL
		",
		server_ids,
		cluster_ids,
	)
	.await?
	.into_iter()
	.map(|(server_id,)| server_id)
	.collect::<Vec<_>>();

	Ok(server_ids)
}
