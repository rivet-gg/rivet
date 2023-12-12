use std::collections::HashMap;

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Cluster {
	name_id: String,
	datacenters: HashMap<String, Datacenter>,
}

#[derive(Deserialize)]
struct Datacenter {
	display_name: String,
	hardware: Vec<Hardware>,
	provider: Provider,
	provider_datacenter_name: String,
	pools: HashMap<PoolType, Pool>,
	drain_timeout: u64,
}

#[derive(Deserialize)]
enum Provider {
	#[serde(rename = "linode")]
	Linode,
}

impl From<Provider> for backend::cluster::Provider {
	fn from(value: Provider) -> backend::cluster::Provider {
		match value {
			Provider::Linode => backend::cluster::Provider::Linode,
		}
	}
}

#[derive(Deserialize)]
struct Hardware {
	name: String,
}

impl From<Hardware> for backend::cluster::Hardware {
	fn from(value: Hardware) -> backend::cluster::Hardware {
		backend::cluster::Hardware {
			provider_hardware: value.name,
		}
	}
}

#[derive(Deserialize)]
struct Pool {
	desired_count: u32,
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
enum PoolType {
	#[serde(rename = "job")]
	Job,
	#[serde(rename = "gg")]
	Gg,
	#[serde(rename = "ats")]
	Ats,
}

impl From<PoolType> for backend::cluster::PoolType {
	fn from(value: PoolType) -> backend::cluster::PoolType {
		match value {
			PoolType::Job => backend::cluster::PoolType::Job,
			PoolType::Gg => backend::cluster::PoolType::Gg,
			PoolType::Ats => backend::cluster::PoolType::Ats,
		}
	}
}

#[tracing::instrument]
pub async fn run_from_env() -> GlobalResult<()> {
	let pools = rivet_pools::from_env("cluster-default-update").await?;
	let client =
		chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("cluster-default-update");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = OperationContext::new(
		"cluster-default-update".into(),
		std::time::Duration::from_secs(60),
		rivet_connection::Connection::new(client, pools, cache),
		Uuid::new_v4(),
		Uuid::new_v4(),
		util::timestamp::now(),
		util::timestamp::now(),
		(),
		Vec::new(),
	);

	// Read config from env
	let Some(config_json) = std::env::var("RIVET_DEFAULT_CLUSTER_CONFIG").ok() else {
		tracing::warn!("no cluster config set in namespace config");
		return Ok(());
	};
	let config = serde_json::from_str::<Cluster>(&config_json)?;

	let cluster_id = util::env::default_cluster_id();

	let cluster_res = op!([ctx] cluster_get {
		cluster_ids: vec![cluster_id.into()],
	})
	.await?;

	if cluster_res.clusters.is_empty() {
		create_cluster(&ctx, cluster_id, config).await?;
	} else {
		tracing::warn!("default cluster already created, updating");
		update_cluster(&ctx, cluster_id, config).await?;
	}

	Ok(())
}

async fn update_cluster(
	ctx: &OperationContext<()>,
	cluster_id: Uuid,
	config: Cluster,
) -> GlobalResult<()> {
	for (name_id, datacenter) in config.datacenters {
		let datacenter_id = Uuid::new_v4();
		msg!([ctx] @wait cluster::msg::datacenter_update(datacenter_id) {
			config: Some(backend::cluster::Datacenter {
				datacenter_id: Some(datacenter_id.into()),
				cluster_id: Some(cluster_id.into()),
				name_id,
				display_name: datacenter.display_name,

				hardware: datacenter.hardware.into_iter().map(Into::into).collect::<Vec<_>>(),

				provider: Into::<backend::cluster::Provider>::into(datacenter.provider) as i32,
				provider_datacenter_id: datacenter.provider_datacenter_name,

				pools: datacenter.pools.into_iter().map(|(pool_type, pool)| {
					backend::cluster::Pool {
						pool_type: Into::<backend::cluster::PoolType>::into(pool_type) as i32,
						desired_count: pool.desired_count,
					}
				}).collect::<Vec<_>>(),
				drain_timeout: datacenter.drain_timeout,
			}),
		})
		.await?;
	}

	Ok(())
}

async fn create_cluster(
	ctx: &OperationContext<()>,
	cluster_id: Uuid,
	config: Cluster,
) -> GlobalResult<()> {
	msg!([ctx] cluster::msg::create(cluster_id) -> cluster::msg::create_complete {
		cluster_id: Some(cluster_id.into()),
		name_id: config.name_id.clone(),
		owner_team_id: None,
	})
	.await?;

	for (name_id, datacenter) in config.datacenters {
		let datacenter_id = Uuid::new_v4();
		msg!([ctx] @wait cluster::msg::datacenter_create(datacenter_id) {
			config: Some(backend::cluster::Datacenter {
				datacenter_id: Some(datacenter_id.into()),
				cluster_id: Some(cluster_id.into()),
				name_id,
				display_name: datacenter.display_name,

				hardware: datacenter.hardware.into_iter().map(Into::into).collect::<Vec<_>>(),

				provider: Into::<backend::cluster::Provider>::into(datacenter.provider) as i32,
				provider_datacenter_id: datacenter.provider_datacenter_name,

				pools: datacenter.pools.into_iter().map(|(pool_type, pool)| {
					backend::cluster::Pool {
						pool_type: Into::<backend::cluster::PoolType>::into(pool_type) as i32,
						desired_count: pool.desired_count,
					}
				}).collect::<Vec<_>>(),
				drain_timeout: datacenter.drain_timeout,
			}),
		})
		.await?;
	}

	Ok(())
}
