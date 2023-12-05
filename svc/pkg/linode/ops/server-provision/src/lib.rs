use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;
use serde_json::json;
use reqwest::header;
use openssl::{pkey::PKey, rsa::Rsa};

struct Server {
	pub provider_datacenter_id: String,
	pub pool_type: backend::cluster::PoolType,
	pub name: String,
	pub provider_hardware: String,
	pub vlan_ip: Ipv4Addr,
	pub volumes: HashMap<String, ServerVolume>,
	pub tags: Vec<String>,
}

pub struct ServerVolume {
	size: usize,
}

#[operation(name = "linode-server-provision")]
pub async fn handle(
	ctx: OperationContext<linode::server_provision::Request>,
) -> GlobalResult<linode::server_provision::Response> {
	let server_id = unwrap!(ctx.server_id).as_uuid();
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(ctx.pool_type));

	let api_token = util::env::read_secret(&["linode", "terraform", "token"]).await?;

	// Choose best candidate from cluster config
	let provider_hardware = ;

	let ns = util::env::namespace();
	let name = format!("{ns}-{provider_datacenter_id}-{pool_type}-{server_id}");
	
	// Find next available vlan index
	let vlan_addr_range = match pool_type {
		backend::cluster::PoolType::Job => util::net::job::vlan_addr_range(),
		backend::cluster::PoolType::Gg => util::net::gg::vlan_addr_range(),
		backend::cluster::PoolType::As => util::net::ats::vlan_addr_range(),
		
	};
	let max_idx = vlan_addr_range.clone().count();
	let (network_index,) = sql_fetch_one!(
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
		pool_type,
		server_idx
	)
	.await?;
	let vlan_ip = unwrap!(vlan_addr_range.nth(network_idx as usize));

	// None configured
	volumes = vec![];

	let firewall_inbound = match pool_type {
		backend::cluster::PoolType::Job => util::net::job::FIREWALL,
		backend::cluster::PoolType::Gg => util::net::gg::FIREWALL,
		backend::cluster::PoolType::As => util::net::ats::FIREWALL,
	};

	let tags = vec![
		// HACK: Linode requires tags to be > 3 characters. We extend the namespace to make sure it
		// meets the minimum length requirement.
		format!("rivet-{ns}"),
		format!("{ns}-{provider_datacenter_id}"),
		format!("{ns}-{pool_type}"),
		format!("{ns}-{provider_datacenter_id}-{pool_type}"),
	];

	region_vlan = util::net::region::vlan_addr_range();
	prefix_len = region_vlan.prefix_len();

	// Build HTTP client
	let mut headers = header::HeaderMap::new();
	headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(api_token)?);
	let client = reqwest::Client::builder().default_headers(headers).build()?;

	create_ssh_key(&client, ).await?;

	Ok(linode::server_provision::Response {
		provider_server_id: todo!(),
	})
}

async fn create_ssh_key(client: &request::Client, ) -> GlobalResult<()> {
	let private_key_openssh = util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let private_key = Rsa::private_key_from_pem(private_key_openssh.as_bytes())?;

    // Extract the public key
    let public_key = PKey::from_rsa(private_key)?;
    let public_key_pem = public_key.public_key_to_pem()?;
	let public_key_str = str::from_utf8(public_key_pem.as_slice())?.trim();

	let res = client
		.post("https://api.linode.com/v4/profile/sshkeys")
		.header("content-type", "application/json")
		.body(json!({
			"label": server.name,
			"ssh_key": public_key_str,
		}))
		.send()
		.await?
		.json::<VerifyResponse>()
		.await?;
	
	Ok(())
}