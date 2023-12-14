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

	let name = util_cluster::full_server_name(&provider_datacenter_id, pool_type, server_id);

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

	// Write SSH key id
	sql_execute!(
		[ctx, &crdb]
		"
		INSERT INTO db_cluster.linode_misc (
			server_id,
			ssh_key_id
		)
		VALUES ($1, $2)
		",
		server_id,
		ssh_key_res.id as i64,
	)
	.await?;

	let create_instance_res = create_instance(&client, &server, &ssh_key_res.public_key).await?;
	let linode_id = create_instance_res.id;

	// Write linode id
	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.linode_misc
		SET linode_id = $2
		WHERE server_id = $1
		",
		server_id,
		linode_id as i64,
	)
	.await?;

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

	// Write firewall id
	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.linode_misc
		SET firewall_id = $2
		WHERE server_id = $1
		",
		server_id,
		firewall_res.id as i64,
	)
	.await?;

	boot_instance(&client, linode_id).await?;

	let public_ip = get_public_ip(&client, linode_id).await?;

	Ok(linode::server_provision::Response {
		provider_server_id: linode_id.to_string(),
		public_ip: public_ip.to_string(),
	})
}
