use std::net::Ipv4Addr;

use proto::backend::{cluster::PoolType, pkg::*};
use reqwest::header;
use rivet_operation::prelude::*;

mod api;
use api::*;

struct ServerCtx {
	provider_datacenter_id: String,
	pool_type: PoolType,
	name: String,
	provider_hardware: String,
	vlan_ip: Ipv4Addr,
	tags: Vec<String>,
	firewall_inbound: Vec<util::net::FirewallRule>,
}

#[operation(name = "linode-server-provision")]
pub async fn handle(
	ctx: OperationContext<linode::server_provision::Request>,
) -> GlobalResult<linode::server_provision::Response> {
	let cluster_id = unwrap!(ctx.cluster_id);
	let server_id = unwrap!(ctx.server_id).as_uuid();
	let pool_type = unwrap!(PoolType::from_i32(ctx.pool_type));

	let cluster_config_res = op!([ctx] cluster_config_get {
		cluster_ids: vec![cluster_id],
	})
	.await?;
	let cluster_config = unwrap!(cluster_config_res.configs.first());
	let datacenter = unwrap!(cluster_config
		.datacenters
		.iter()
		.find(|dc| dc.datacenter_id == ctx.datacenter_id));
	let provider_datacenter_id = datacenter.provider_datacenter_id.clone();

	// TODO: Choose best candidate
	let provider_hardware = unwrap!(datacenter.hardware.first())
		.provider_hardware
		.clone();

	// Find next available vlan index
	let mut vlan_addr_range = match pool_type {
		PoolType::Job => util::net::job::vlan_addr_range(),
		PoolType::Gg => util::net::gg::vlan_addr_range(),
		PoolType::Ats => util::net::ats::vlan_addr_range(),
	};
	let max_idx = vlan_addr_range.count() as i64;
	let (network_idx,) = sql_fetch_one!(
		[ctx, (i64,)]
		"
		WITH
			get_next_network_idx AS (
				SELECT idx
				FROM generate_series(0, $1) AS s(idx)
				WHERE NOT EXISTS (
					SELECT 1
					FROM db_cluster.servers
					WHERE
						pool_type = $2 AND
						network_idx = idx
				)
				LIMIT 1
			),
			update_network_idx AS (
				UPDATE db_cluster.servers
				SET network_idx = (SELECT idx FROM get_next_network_idx)
				WHERE server_id = $3
				RETURNING 1
			)
		SELECT idx FROM get_next_network_idx
		",
		max_idx,
		ctx.pool_type as i64,
		server_id
	)
	.await?;
	let vlan_ip = unwrap!(vlan_addr_range.nth(network_idx as usize));

	let firewall_inbound = match pool_type {
		PoolType::Job => util::net::job::firewall(),
		PoolType::Gg => util::net::gg::firewall(),
		PoolType::Ats => util::net::ats::firewall(),
	};

	let pool_type_str = match pool_type {
		PoolType::Job => "job",
		PoolType::Gg => "gg",
		PoolType::Ats => "ats",
	};

	let ns = util::env::namespace();
	let name = format!("{ns}-{provider_datacenter_id}-{pool_type_str}-{server_id}");

	let tags = vec![
		// HACK: Linode requires tags to be > 3 characters. We extend the namespace to make sure it
		// meets the minimum length requirement.
		format!("rivet-{ns}"),
		format!("{ns}-{provider_datacenter_id}"),
		format!("{ns}-{pool_type_str}"),
		format!("{ns}-{provider_datacenter_id}-{pool_type_str}"),
	];

	let server = ServerCtx {
		provider_datacenter_id,
		pool_type,
		name,
		provider_hardware,
		vlan_ip,
		tags,
		firewall_inbound,
	};

	// Build HTTP client
	let api_token = util::env::read_secret(&["linode", "terraform", "token"]).await?;
	let mut headers = header::HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		header::HeaderValue::from_str(&api_token)?,
	);
	headers.insert(
		header::CONTENT_TYPE,
		header::HeaderValue::from_static("application/json"),
	);
	let client = reqwest::Client::builder()
		.default_headers(headers)
		.build()?;

	let ssh_key = create_ssh_key(&client, &server).await?;
	let create_instance_res = create_instance(&client, ns, &ssh_key, &server).await?;
	let linode_id = create_instance_res.id;
	let create_disks_res = create_disks(
		&client,
		&ssh_key,
		linode_id,
		create_instance_res.specs.disk,
		&server,
	)
	.await?;
	create_instance_config(&client, ns, linode_id, &create_disks_res, &server).await?;
	create_firewall(&client, linode_id, &server).await?;

	Ok(linode::server_provision::Response {
		provider_server_id: linode_id.to_string(),
	})
}
