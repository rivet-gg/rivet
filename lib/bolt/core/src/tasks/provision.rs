use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::*;
use futures_util::stream::StreamExt;
use rivet_api::{
	apis::{admin_clusters_api, admin_clusters_datacenters_api, configuration},
	models,
};
// use proto::backend::{self, pkg::*};
// use rivet_operation::prelude::*;
use serde::Deserialize;
use tokio::{
	fs,
	process::Command,
	sync::{Mutex, Semaphore},
	task::JoinSet,
};
use uuid::Uuid;

use crate::{
	config::{
		self,
		service::{ComponentClass, RuntimeKind},
	},
	context::{
		BuildContext, BuildOptimization, ProjectContext, RunContext, ServiceBuildPlan,
		ServiceContext,
	},
	dep::{
		self, cargo,
		k8s::gen::{ExecServiceContext, ExecServiceDriver},
	},
	tasks,
	utils::{self},
};

#[derive(Deserialize)]
struct Cluster {
	name_id: String,
	datacenters: HashMap<String, Datacenter>,
}

#[derive(Deserialize)]
struct Datacenter {
	datacenter_id: Uuid,
	display_name: String,
	provider: Provider,
	provider_datacenter_name: String,
	pools: HashMap<PoolType, Pool>,
	build_delivery_method: BuildDeliveryMethod,
	drain_timeout: u64,
}

#[derive(Deserialize)]
enum Provider {
	#[serde(rename = "linode")]
	Linode,
}

#[derive(Deserialize)]
struct Pool {
	hardware: Vec<Hardware>,
	desired_count: u32,
	max_count: u32,
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

#[derive(Deserialize)]
struct Hardware {
	name: String,
}

#[derive(Deserialize)]
enum BuildDeliveryMethod {
	#[serde(rename = "traffic_server")]
	TrafficServer,
	#[serde(rename = "s3_direct")]
	S3Direct,
}

pub async fn default_cluster_create(ctx: &ProjectContext) -> Result<()> {
	// Read config from env
	let Some(config_json) = util::env::var("RIVET_DEFAULT_CLUSTER_CONFIG").ok() else {
		tracing::warn!("no cluster config set in namespace config");
		return Ok(());
	};
	let config = serde_json::from_str::<Cluster>(&config_json)?;

	let api_admin_token = ctx.read_secret(&["rivet", "api_admin", "token"]).await?;

	let req_config = configuration::Configuration {
		base_path: ctx.origin_api(),
		bearer_access_token: Some(api_admin_token),
		..Default::default()
	};

	// Get the default cluster
	let cluster_res =
		admin_clusters_api::admin_clusters_get(&req_config, &Uuid::nil().to_string()).await;

	let datacenter_list_res =
		admin_clusters_datacenters_api::admin_clusters_datacenters_get_datacenters(
			&req_config,
			&Uuid::nil().to_string(),
		)
		.await?;

	let datacenters = datacenter_list_res.datacenters;

	// If the default cluster doesn't exist, create it
	if cluster_res.is_err() {
		rivet_term::status::progress("Creating default cluster", "");

		admin_clusters_api::admin_clusters_create(
			&req_config,
			models::AdminClustersCreateRequest {
				name_id: "default".to_string(),
				owner_team_id: None,
			},
		)
		.await?;
	}

	// Delete any datacenters that aren't in the config

	// Go through each datacenter in the config and update it if needed
	for (name_id, datacenter) in config.datacenters {
		let existing_datacenter = datacenters
			.iter()
			.any(|dc| dc.datacenter_id == datacenter.datacenter_id);

		// Update existing datacenter
		if existing_datacenter {
			let new_pools = datacenter
				.pools
				.into_iter()
				.map(|(pool_type, pool)| {
					let desired_count = match pool_type {
						PoolType::Ats => Some(pool.desired_count),
						PoolType::Job | PoolType::Gg => {
							// TODO: Add autoscaler to namespace
							let use_autoscaler = false;
							if use_autoscaler {
								None
							} else {
								Some(pool.desired_count)
							}
						}
					};

					clusters::msg::datacenter_update::PoolUpdate {
						pool_type: Into::<backend::cluster::PoolType>::into(pool_type) as i32,
						hardware: pool
							.hardware
							.into_iter()
							.map(Into::into)
							.collect::<Vec<_>>(),
						desired_count,
						max_count: Some(pool.max_count),
					}
				})
				.collect::<Vec<_>>();

			msg!([ctx] @wait cluster::msg::datacenter_update(datacenter.datacenter_id) {
				datacenter_id: datacenter_id_proto,
				pools: new_pools,
				// Convert from seconds to ms
				drain_timeout: Some(datacenter.drain_timeout * 1000),
			})
			.await?;
		}
		// Create new datacenter
		else {
			msg!([ctx] @wait cluster::msg::datacenter_create(datacenter.datacenter_id) {
				config: Some(backend::cluster::Datacenter {
					datacenter_id: datacenter_id_proto,
					cluster_id: Some(cluster_id.into()),
					name_id,
					display_name: datacenter.display_name,

					provider: Into::<backend::cluster::Provider>::into(datacenter.provider) as i32,
					provider_datacenter_id: datacenter.provider_datacenter_name,
					provider_api_token: None,

					pools: datacenter.pools.into_iter().map(|(pool_type, pool)| {
						backend::cluster::Pool {
							pool_type: Into::<backend::cluster::PoolType>::into(pool_type) as i32,
							hardware: pool.hardware.into_iter().map(Into::into).collect::<Vec<_>>(),
							desired_count: pool.desired_count,
							max_count: pool.max_count,
						}
					}).collect::<Vec<_>>(),

					build_delivery_method: Into::<backend::cluster::BuildDeliveryMethod>::into(datacenter.build_delivery_method) as i32,
					drain_timeout: datacenter.drain_timeout,
				}),
			})
			.await?;
		}

		// TODO: Both this message and datacenter-create/datacenter-update (above) publish datacenter-scale.
		// This results in double provisioning until datacenter-scale is published again, cleaning up the
		// excess.
		// Taint datacenter
		if taint {
			msg!([ctx] @wait cluster::msg::datacenter_taint(datacenter.datacenter_id) {
				datacenter_id: datacenter_id_proto,
			})
			.await?;
		}
	}

	Ok(())
}
