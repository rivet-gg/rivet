use std::net::{Ipv4Addr, Ipv6Addr};

pub struct FirewallRule {
	pub label: String,
	pub ports: Port,
	pub protocol: Protocol,
	pub inbound_ipv4_cidr: Vec<Ipv4CidrAddr>,
	pub inbound_ipv6_cidr: Vec<Ipv6CidrAddr>,
}

pub enum Port {
	Single(u16),
	Range(u16, u16),
}

pub enum Protocol {
	Tcp,
	Udp,
	Icmp,

	// Linode only
	Ipencap,

	// Vultr only
	Gre,
	Esp,
	Ah,
}

impl Protocol {
	pub fn as_uppercase(&self) -> &str {
		match self {
			Protocol::Tcp => "TCP",
			Protocol::Udp => "UDP",
			Protocol::Icmp => "ICMP",
			Protocol::Ipencap => "IPENCAP",
			Protocol::Gre => "GRE",
			Protocol::Esp => "ESP",
			Protocol::Ah => "AH",
		}
	}
}

pub struct Ipv4CidrAddr(Ipv4Addr, u8);

impl Ipv4CidrAddr {
	pub fn subnet(&self) -> Ipv4Addr {
		self.0
	}

	pub fn subnet_size(&self) -> u8 {
		self.1
	}
}

impl std::fmt::Display for Ipv4CidrAddr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Ipv4CidrAddr(subnet, subnet_size) = self;

		write!(f, "{}", subnet)?;
		write!(f, "{}", subnet_size)
	}
}

pub struct Ipv6CidrAddr(Ipv6Addr, u8);

impl Ipv6CidrAddr {
	pub fn subnet(&self) -> Ipv6Addr {
		self.0
	}

	pub fn subnet_size(&self) -> u8 {
		self.1
	}
}

impl std::fmt::Display for Ipv6CidrAddr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Ipv6CidrAddr(subnet, subnet_size) = self;
		
		write!(f, "{}", subnet)?;
		write!(f, "{}", subnet_size)
	}
}


pub fn default_firewall() -> FirewallRule {
	FirewallRule {
		label: "ssh".into(),
		ports: Port::Single(22),
		protocol: Protocol::Tcp,
		inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
		inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
	}
}

pub mod gg {
	use std::net::{Ipv4Addr, Ipv6Addr};

	use ipnet::{Ipv4AddrRange, Ipv4Net};

	use super::{default_firewall, job, Protocol, FirewallRule, Ipv4CidrAddr, Ipv6CidrAddr, Port};

	pub fn vlan_ip_net() -> Ipv4Net {
		Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 0), 26).unwrap()
	}

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		vlan_ip_net().hosts()
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![
			default_firewall(),
			// HTTP(S)
			FirewallRule {
				label: "http-tcp".into(),
				ports: Port::Single(80),
				protocol: Protocol::Tcp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			FirewallRule {
				label: "http-udp".into(),
				ports: Port::Single(80),
				protocol: Protocol::Udp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			FirewallRule {
				label: "https-tcp".into(),
				ports: Port::Single(443),
				protocol: Protocol::Tcp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			FirewallRule {
				label: "https-udp".into(),
				ports: Port::Single(443),
				protocol: Protocol::Udp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			// Dynamic TCP
			FirewallRule {
				label: "dynamic-tcp".into(),
				ports: Port::Range(job::MIN_INGRESS_PORT_TCP, job::MAX_INGRESS_PORT_TCP),
				protocol: Protocol::Tcp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			// Dynamic UDP
			FirewallRule {
				label: "dynamic-udp".into(),
				ports: Port::Range(job::MIN_INGRESS_PORT_UDP, job::MAX_INGRESS_PORT_UDP),
				protocol: Protocol::Udp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
		]
	}
}

pub mod ats {
	use std::net::{Ipv4Addr};

	use ipnet::{Ipv4AddrRange, Ipv4Net};

	use super::{default_firewall, FirewallRule};

	pub fn vlan_ip_net() -> Ipv4Net {
		Ipv4Net::new(Ipv4Addr::new(10, 0, 0, 64), 26).unwrap()
	}

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		vlan_ip_net().hosts()
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![default_firewall()]
	}
}

pub mod job {
	use std::net::{Ipv4Addr, Ipv6Addr};

	use ipnet::Ipv4AddrRange;

	use super::{default_firewall, Protocol, FirewallRule, Ipv4CidrAddr, Ipv6CidrAddr, Port};

	// Port ranges for the load balancer hosts
	// 20000-26000 are for traffic from gg on LAN
	// 26000-31999 is for host networking only
	pub const MIN_INGRESS_PORT_TCP: u16 = 20000;
	pub const MIN_HOST_PORT_TCP: u16 = 26000;
	pub const MAX_INGRESS_PORT_TCP: u16 = 31999;
	pub const MIN_INGRESS_PORT_UDP: u16 = 20000;
	pub const MIN_HOST_PORT_UDP: u16 = 26000;
	pub const MAX_INGRESS_PORT_UDP: u16 = 31999;

	pub fn vlan_addr_range() -> Ipv4AddrRange {
		Ipv4AddrRange::new(Ipv4Addr::new(10, 0, 4, 1), Ipv4Addr::new(10, 0, 255, 254))
	}

	pub fn firewall() -> Vec<FirewallRule> {
		vec![
			default_firewall(),
			// Ports available to Nomad jobs using the host network
			FirewallRule {
				label: "nomad-host-tcp".into(),
				ports: Port::Range(MIN_HOST_PORT_TCP, MAX_INGRESS_PORT_TCP),
				protocol: Protocol::Tcp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
			FirewallRule {
				label: "nomad-host-udp".into(),
				ports: Port::Range(MIN_HOST_PORT_UDP, MAX_INGRESS_PORT_UDP),
				protocol: Protocol::Udp,
				inbound_ipv4_cidr: vec![Ipv4CidrAddr(Ipv4Addr::new(0, 0, 0, 0), 0)],
				inbound_ipv6_cidr: vec![Ipv6CidrAddr(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 0)],
			},
		]
	}
}
