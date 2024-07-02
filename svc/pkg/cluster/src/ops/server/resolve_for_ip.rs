use std::net::{IpAddr, Ipv4Addr};

use chirp_workflow::prelude::*;

pub struct Input {
	pub ips: Vec<Ipv4Addr>,
}

pub struct Output {
	pub servers: Vec<Server>,
}

#[derive(sqlx::FromRow)]
pub struct Server {
	pub server_id: Uuid,
	pub public_ip: IpAddr,
}

#[operation]
pub async fn cluster_server_resolve_for_ip(
	ctx: &OperationCtx,
	input: &Input,
) -> GlobalResult<Output> {
	let servers = sql_fetch_all!(
		[ctx, Server]
		"
		SELECT
			server_id,
			datacenter_id,
			pool_type,
			vlan_ip,
			public_ip,
			cloud_destroy_ts
		FROM db_cluster.servers
		WHERE server_id = ANY($1)
		",
		input.ips
			.iter()
			.cloned()
			.map(IpAddr::V4)
			.collect::<Vec<_>>(),
	)
	.await?;

	Ok(Output { servers })
}
