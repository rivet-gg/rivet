use chirp_workflow::prelude::*;

use crate::ops::datacenter::get::DatacenterRow;
use crate::types::Datacenter;

#[derive(Debug)]
pub struct Input {
	pub labels: Vec<u16>,
}

#[derive(Debug)]
pub struct Output {
	pub datacenters: Vec<Datacenter>,
}

#[operation]
pub async fn cluster_datacenter_get_for_label(
	ctx: &OperationCtx,
	input: &Input,
) -> GlobalResult<Output> {
	let datacenters = ctx
		.cache()
		.fetch_all_json("cluster.datacenters_get_for_label", input.labels.clone(), {
			let ctx = ctx.clone();
			move |mut cache, labels| {
				let ctx = ctx.clone();
				async move {
					let dcs = get_dcs(ctx, labels).await?;
					for dc in dcs {
						cache.resolve(&dc.label(), dc);
					}

					Ok(cache)
				}
			}
		})
		.await?;

	Ok(Output { datacenters })
}

async fn get_dcs(ctx: OperationCtx, labels: Vec<u16>) -> GlobalResult<Vec<Datacenter>> {
	let dc_rows = sql_fetch_all!(
		[ctx, DatacenterRow]
		"
		SELECT
			datacenter_id,
			cluster_id,
			name_id,
			display_name,
			provider,
			provider_datacenter_id,
			provider_api_token,
			pools2,
			build_delivery_method,
			prebakes_enabled,
			create_ts,
			guard_public_hostname_dns_parent,
			guard_public_hostname_static
		FROM db_cluster.datacenters@datacenter_label_idx
		WHERE label = ANY($1)
		",
		labels.into_iter().map(|x| x.to_be_bytes()).collect::<Vec<_>>(),
	)
	.await?;

	dc_rows
		.into_iter()
		.map(|row| row.into_datacenter(ctx.config()))
		.collect::<GlobalResult<Vec<_>>>()
}
