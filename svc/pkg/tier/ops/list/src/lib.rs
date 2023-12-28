use std::collections::HashMap;

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use util_linode::api;

// See pkr/static/nomad-config.hcl.tpl client.reserved
const RESERVE_SYSTEM_CPU: u64 = 500;
const RESERVE_SYSTEM_MEMORY: u64 = 512;

// See module.traefik_job resources
const RESERVE_LB_CPU: u64 = 1500;
const RESERVE_LB_MEMORY: u64 = 512;

const RESERVE_CPU: u64 = RESERVE_SYSTEM_CPU + RESERVE_LB_CPU;
const RESERVE_MEMORY: u64 = RESERVE_SYSTEM_MEMORY + RESERVE_LB_MEMORY;

struct GameNodeConfig {
	cpu_cores: u64,
	cpu: u64,
	memory: u64,
	disk: u64,
	bandwidth: u64,
}

impl GameNodeConfig {
	fn cpu_per_core(&self) -> u64 {
		self.cpu / self.cpu_cores
	}

	fn memory_per_core(&self) -> u64 {
		self.memory / self.cpu_cores
	}

	fn disk_per_core(&self) -> u64 {
		self.disk / self.cpu_cores
	}

	fn bandwidth_per_core(&self) -> u64 {
		self.bandwidth / self.cpu_cores
	}
}

#[operation(name = "tier-list")]
async fn handle(ctx: OperationContext<tier::list::Request>) -> GlobalResult<tier::list::Response> {
	let datacenters_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: ctx.region_ids.clone(),
	})
	.await?;

	// Build HTTP client
	let client = util_linode::Client::new().await?;

	// Get hardware stats from linode and cache
	let instance_types_res = ctx
		.cache()
		.ttl(util::duration::days(1))
		.fetch_one_proto("instance_type", "linode", {
			let client = client.clone();
			move |mut cache, key| {
				let client = client.clone();
				async move {
					let api_res = api::list_instance_types(&client).await?;

					cache.resolve(
						&key,
						tier::list::CacheInstanceTypes {
							instance_types: api_res.into_iter().map(Into::into).collect::<Vec<_>>(),
						},
					);

					Ok(cache)
				}
			}
		})
		.await?;
	let instance_types = unwrap!(instance_types_res)
		.instance_types
		.into_iter()
		.map(|ty| (ty.id.clone(), ty))
		.collect::<HashMap<_, _>>();

	let regions = datacenters_res
		.datacenters
		.iter()
		.map(|datacenter| {
			let job_pool = unwrap!(
				datacenter
					.pools
					.iter()
					.find(|pool| pool.pool_type == backend::cluster::PoolType::Job as i32),
				"no job pool"
			);
			let hardware = &unwrap!(job_pool.hardware.first(), "no hardware").provider_hardware;
			let instance_type = unwrap!(
				instance_types.get(hardware),
				"datacenter hardware stats not found"
			);

			Ok(tier::list::response::Region {
				region_id: datacenter.datacenter_id,
				tiers: vec![
					generate_tier(instance_type, "basic-4d1", 4, 1),
					generate_tier(instance_type, "basic-2d1", 2, 1),
					generate_tier(instance_type, "basic-1d1", 1, 1),
					generate_tier(instance_type, "basic-1d2", 1, 2),
					generate_tier(instance_type, "basic-1d4", 1, 4),
					generate_tier(instance_type, "basic-1d8", 1, 8),
					generate_tier(instance_type, "basic-1d16", 1, 16),
				],
			})
		})
		.collect::<GlobalResult<Vec<_>>>()?;

	Ok(tier::list::Response { regions })
}

/// Returns the default game node config.
fn get_game_node_config(instance_type: &tier::list::CacheInstanceType) -> GameNodeConfig {
	// Multiply config for 2 core, 4 GB to scale up to the 4 core, 8 GB
	// plan
	let mut config = GameNodeConfig {
		cpu_cores: instance_type.vcpus,
		// DigitalOcean: 7,984
		// Linode: 7,996
		cpu: 7900,
		memory: instance_type.memory,
		disk: instance_type.disk,
		bandwidth: instance_type.network_out * 1000,
	};

	// Remove reserved resources
	config.cpu -= RESERVE_CPU;
	config.memory -= RESERVE_MEMORY;

	config
}

fn generate_tier(
	instance_type: &tier::list::CacheInstanceType,
	name: &str,
	numerator: u64,
	denominator: u64,
) -> backend::region::Tier {
	let c = get_game_node_config(instance_type);

	backend::region::Tier {
		tier_name_id: name.into(),
		rivet_cores_numerator: numerator as u32,
		rivet_cores_denominator: denominator as u32,
		cpu: c.cpu_per_core() * numerator / denominator,
		memory: c.memory_per_core() * numerator / denominator,
		// Allow oversubscribing memory by 50% of the reserved
		// memory
		memory_max: u64::min(
			(c.memory_per_core() * numerator / denominator) * 3 / 2,
			c.memory,
		),
		disk: c.disk_per_core() * numerator / denominator,
		bandwidth: c.bandwidth_per_core() * numerator / denominator,
	}
}
