use chirp_workflow::prelude::*;

use crate::{
	types::Plan,
	util::{api, client},
};

#[derive(Debug)]
pub struct Input {
	pub hardware_ids: Vec<String>,
}

#[derive(Debug)]
pub struct Output {
	pub plans: Vec<Plan>,
}

#[operation]
pub async fn linode_plan_get(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	// Build HTTP client
	let client = client::Client::new(None).await?;

	// Get hardware stats from vultr and cache
	let plans_res = ctx
		.cache()
		.ttl(util::duration::days(1))
		.fetch_one_json("plans", "vultr", {
			let client = client.clone();
			move |mut cache, key| {
				let client = client.clone();
				async move {
					let api_res = api::list_plans(&client).await?;

					cache.resolve(
						&key,
						api_res
							.into_iter()
							.map(Into::<Plan>::into)
							.collect::<Vec<_>>(),
					);

					Ok(cache)
				}
			}
		})
		.await?;

	// Filter by hardware
	let plans = unwrap!(plans_res)
		.into_iter()
		.filter(|ty| input.hardware_ids.iter().any(|h| h == &ty.hardware_id))
		.collect::<Vec<_>>();

	Ok(Output { plans })
}
