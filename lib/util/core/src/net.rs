pub struct FirewallRule {
	pub label: String,
	pub ports: String,
	pub protocol: String,
	pub inbound_ipv4_cidr: Vec<String>,
	pub inbound_ipv6_cidr: Vec<String>,
}

pub mod region {
	use std::net::Ipv4Addr;

	use ipnet::Ipv4Net;

	pub fn vlan_ip_net() -> Ipv4Net {
		Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0), 16).unwrap()
	}
}

pub mod gg {
	use std::net::Ipv4Addr;

	use ipnet::{Ipv4AddrRange, Ipv4Net};

	use super::FirewallRule;

	pub fn vlan_ip_net() -> Ipv4Net {
		Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0), 26).unwrap()
	}

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		vlan_ip_net().hosts()
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![
			// HTTP(S)
			FirewallRule {
				label: "http-tcp".into(),
				ports: "80".into(),
				protocol: "tcp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			FirewallRule {
				label: "http-udp".into(),
				ports: "80".into(),
				protocol: "udp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			FirewallRule {
				label: "https-tcp".into(),
				ports: "443".into(),
				protocol: "tcp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			FirewallRule {
				label: "https-udp".into(),
				ports: "443".into(),
				protocol: "udp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			// Dynamic TCP
			FirewallRule {
				label: "dynamic-tcp".into(),
				ports: "20000-31999".into(),
				protocol: "tcp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			// Dynamic UDP
			FirewallRule {
				label: "dynamic-udp".into(),
				ports: "20000-31999".into(),
				protocol: "udp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
		]
	}
}

pub mod ats {
	use std::net::Ipv4Addr;

	use ipnet::{Ipv4AddrRange, Ipv4Net};

	use super::FirewallRule;

	pub fn vlan_ip_net() -> Ipv4Net {
		Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 64), 26).unwrap()
	}

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		vlan_ip_net().hosts()
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![]
	}
}

// 10.0.64-10.0.4.0 reserved for more services

pub mod job {
	use std::net::Ipv4Addr;

	use ipnet::Ipv4AddrRange;

	use super::FirewallRule;

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		Ipv4AddrRange::new(Ipv4Addr::new(10, 0, 4, 1), Ipv4Addr::new(10, 0, 255, 254))
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![
			// Ports available to Nomad jobs using the host network
			FirewallRule {
				label: "nomad-host-tcp".into(),
				ports: "26000-31999".into(),
				protocol: "tcp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
			FirewallRule {
				label: "nomad-host-udp".into(),
				ports: "26000-31999".into(),
				protocol: "udp".into(),
				inbound_ipv4_cidr: vec!["0.0.0.0/0".into()],
				inbound_ipv6_cidr: vec!["::/0".into()],
			},
		]
	}
}
