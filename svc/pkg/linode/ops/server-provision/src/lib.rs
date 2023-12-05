use std::net::Ipv4Addr;

use proto::backend::{cluster::PoolType, pkg::*};
use rand::Rng;
use reqwest::header;
use rivet_operation::prelude::*;

mod api;
use api::*;

struct ServerCtx {
	provider_datacenter_id: String,
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
	let server_id = unwrap!(ctx.server_id).as_uuid();
	let provider_datacenter_id = ctx.provider_datacenter_id.clone();
	let pool_type = unwrap!(PoolType::from_i32(ctx.pool_type));
	let provider_hardware = unwrap_ref!(ctx.hardware).provider_hardware.clone();

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
				FROM generate_series(0, $2) AS s(idx)
				WHERE NOT EXISTS (
					SELECT 1
					FROM db_cluster.servers
					WHERE
						pool_type = $2 AND
						network_idx = mod(idx + $1, $2)
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
		// Choose a random index to start from for better index spread
		rand::thread_rng().gen_range(0i64..max_idx),
		max_idx,
		ctx.pool_type as i64,
		server_id
	)
	.await?;
	let vlan_ip = unwrap!(vlan_addr_range.nth(network_idx as usize));

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

	let firewall_inbound = match pool_type {
		PoolType::Job => util::net::job::firewall(),
		PoolType::Gg => util::net::gg::firewall(),
		PoolType::Ats => util::net::ats::firewall(),
	};

	// Build context
	let server = ServerCtx {
		provider_datacenter_id,
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

	// Run API calls
	let ssh_key = create_ssh_key(&client, &server).await?;
	let create_instance_res = create_instance(&client, ns, &ssh_key, &server).await?;
	let linode_id = create_instance_res.id;
	let create_disks_res =
		create_disks(&client, &ssh_key, linode_id, create_instance_res.specs.disk).await?;
	create_instance_config(&client, ns, linode_id, &create_disks_res, &server).await?;
	create_firewall(&client, linode_id, &server).await?;
	let public_ip = get_public_ip(&client, linode_id).await?;

	Ok(linode::server_provision::Response {
		provider_server_id: linode_id.to_string(),
		vlan_ip: vlan_ip.to_string(),
		public_ip: public_ip.to_string(),
	})
}
