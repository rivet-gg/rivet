use std::net::Ipv4Addr;

use proto::backend::{cluster::PoolType, pkg::*};
use reqwest::header;
use rivet_operation::prelude::*;

mod api;
use api::*;

struct ServerCtx {
	provider_datacenter_id: String,
	name: String,
	provider_hardware: String,
	vlan_ip: String,
	tags: Vec<String>,
	firewall_inbound: Vec<util::net::FirewallRule>,
}

struct RestResponse {
	linode_id: u64,
	firewall_id: u64,
	public_ip: Ipv4Addr,
}

#[operation(name = "linode-server-provision", timeout = 150)]
pub async fn handle(
	ctx: OperationContext<linode::server_provision::Request>,
) -> GlobalResult<linode::server_provision::Response> {
	let crdb = ctx.crdb().await?;
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();
	let provider_datacenter_id = ctx.provider_datacenter_id.clone();
	let pool_type = unwrap!(PoolType::from_i32(ctx.pool_type));
	let provider_hardware = unwrap_ref!(ctx.hardware).provider_hardware.clone();

	let ns = util::env::namespace();
	let pool_type_str = match pool_type {
		PoolType::Job => "job",
		PoolType::Gg => "gg",
		PoolType::Ats => "ats",
	};

	let name = format!(
		"{}-{server_id}",
		util_cluster::server_name(&provider_datacenter_id, pool_type)
	);

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
		vlan_ip: ctx.vlan_ip.clone(),
		tags,
		firewall_inbound,
	};

	// Build HTTP client
	let api_token = util::env::read_secret(&["linode", "token"]).await?;
	let auth = format!("Bearer {}", api_token,);
	let mut headers = header::HeaderMap::new();
	headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&auth)?);
	let client = reqwest::Client::builder()
		.default_headers(headers)
		.build()?;

	// Create SSH key
	let ssh_key_res = create_ssh_key(&client, &server).await?;

	// Run the rest of the API calls. This is done in an isolated manner so that if any errors occur here,
	// the ssh key id can still be written to database.
	let rest_res = async {
		let create_instance_res =
			create_instance(&client, &server, &ssh_key_res.public_key).await?;
		let linode_id = create_instance_res.id;

		wait_instance_ready(&client, linode_id).await?;

		let create_disks_res = create_disks(
			&client,
			&ssh_key_res.public_key,
			linode_id,
			create_instance_res.specs.disk,
		)
		.await?;

		create_instance_config(&client, &server, linode_id, &create_disks_res).await?;

		let firewall_res = create_firewall(&client, &server, linode_id).await?;

		boot_instance(&client, linode_id).await?;

		let public_ip = get_public_ip(&client, linode_id).await?;

		GlobalResult::Ok(RestResponse {
			linode_id,
			firewall_id: firewall_res.id,
			public_ip,
		})
	}
	.await;

	// Extract firewall_id as `Option`
	let firewall_id = rest_res.as_ref().ok().map(|res| res.firewall_id as i64);

	// These values are used when destroying resources
	sql_execute!(
		[ctx, &crdb]
		"
		INSERT INTO db_cluster.linode_misc (
			server_id,
			ssh_key_id,
			firewall_id
		)
		VALUES ($1, $2, $3)
		",
		server_id,
		ssh_key_res.id as i64,
		firewall_id
	)
	.await?;

	let rest_res = rest_res?;

	Ok(linode::server_provision::Response {
		provider_server_id: rest_res.linode_id.to_string(),
		public_ip: rest_res.public_ip.to_string(),
	})
}
