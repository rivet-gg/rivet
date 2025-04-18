use chirp_workflow::prelude::*;

pub fn install() -> String {
	include_str!("../files/nomad_install.sh").to_string()
}

pub fn configure(config: &rivet_config::Config) -> GlobalResult<String> {
	let provision_config = config.server()?.rivet.provision()?;

	let nomad_server_count = config.server()?.nomad()?.server_count;
	let servers = (0..nomad_server_count)
		.map(|idx| format!("127.0.0.1:{}", 5000 + idx))
		.collect::<Vec<_>>();

	Ok(include_str!("../files/nomad_configure.sh")
		// HACK: Hardcoded to Linode
		.replace("__PUBLIC_IFACE__", "eth0")
		// HACK: Hardcoded to Linode
		.replace("__VLAN_IFACE__", "eth1")
		.replace(
			"__MIN_DYNAMIC_PORT__",
			&provision_config.pools.pegboard.min_lan_port().to_string(),
		)
		.replace(
			"__MAX_DYNAMIC_PORT__",
			&provision_config.pools.pegboard.max_lan_port().to_string(),
		)
		.replace(
			"__SERVER_JOIN__",
			&servers
				.iter()
				.map(|x| format!("\"{x}\""))
				.collect::<Vec<_>>()
				.join(", "),
		)
		.replace(
			"__RESERVE_MEMORY__",
			&server_spec::NOMAD_RESERVE_MEMORY_MIB.to_string(),
		)
		.replace(
			"__GG_VLAN_SUBNET__",
			&provision_config.pools.gg.vlan_ip_net().to_string(),
		)
		.replace(
			"__ATS_VLAN_SUBNET__",
			&provision_config.pools.ats.vlan_ip_net().to_string(),
		))
}
