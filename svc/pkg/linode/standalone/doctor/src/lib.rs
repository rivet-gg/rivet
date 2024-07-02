use proto::backend;
use reqwest::header;
use rivet_operation::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct GetLinodesResponse {
	data: Vec<Linode>,
}

#[derive(Deserialize)]
struct Linode {
	created: chrono::NaiveDateTime,
	label: String,
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(pools: rivet_pools::Pools) -> GlobalResult<()> {
	let client = chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("linode-gc");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = OperationContext::new(
		"linode-gc".into(),
		std::time::Duration::from_secs(60),
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
	);

	let dc_rows = sql_fetch_all!(
		[ctx, (i64, String,)]
		"
		SELECT provider, provider_api_token
		FROM db_cluster.datacenters
		WHERE provider_api_token IS NOT NULL
		",
	)
	.await?
	.into_iter()
	.chain(std::iter::once((
		backend::cluster::Provider::Linode as i64,
		util::env::read_secret(&["linode", "token"]).await?,
	)));

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

	for (provider, api_token) in dc_rows {
		let provider = unwrap!(backend::cluster::Provider::from_i32(provider as i32));

		match provider {
			backend::cluster::Provider::Linode => {
				run_for_linode_account(&ctx, &api_token, &headers).await?
			}
		}
	}

	Ok(())
}

async fn run_for_linode_account(
	ctx: &OperationContext<()>,
	api_token: &str,
	headers: &header::HeaderMap,
) -> GlobalResult<()> {
	// Build HTTP client
	let client =
		util_linode::Client::new_with_headers(Some(api_token.to_string()), headers.clone()).await?;

	let req = client
		.inner()
		.get("https://api.linode.com/v4/linode/instances")
		.query(&[("page_size", 500)]);

	let res = client.request(req, None, false).await?
		.json::<GetLinodesResponse>()
		.await?;

	tracing::info!("{} servers in cluster", res.data.len());

	// Filter out servers younger than 12 hours
	let server_ids = res.data
		.into_iter()
		.filter(|linode| {
			linode.created.and_utc().timestamp_millis() < ctx.ts() - util::duration::hours(12)
		})
		.map(|linode| {
			util::uuid::parse(&linode.label.replacen(
				&format!("{}-", util::env::namespace()),
				"",
				1,
			))
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	// Select servers from linode api call that should be deleted
	let server_rows = sql_fetch_all!(
		[ctx, (Uuid, Uuid)]
		"
		SELECT server_id, datacenter_id
		FROM db_cluster.servers
		WHERE
			server_id = ANY($1) AND
			cloud_destroy_ts IS NOT NULL
		",
		server_ids,
	)
	.await?;

	if !server_rows.is_empty() {
		tracing::info!(
			server_ids=?server_rows.iter().map(|x| x.0).collect::<Vec<_>>(),
			"deleting {} servers", server_rows.len(),
		);
	
		for (server_id, dc_id) in server_rows {
			op!([ctx] linode_server_destroy {
				server_id: Some(server_id.into()),
				datacenter_id: Some(dc_id.into()),
			})
			.await?;
		}
	}


	Ok(())
}
