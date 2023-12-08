use std::{net::Ipv4Addr, env, collections::HashMap};

use chirp_worker::prelude::*;
use proto::backend;
use maplit::hashmap;
use indoc::formatdoc;

pub struct ServerCtx {
	pub provider_datacenter_id: String,
	pub universal_region_str: String,
	pub name: String,
	pub pool_type: backend::cluster::PoolType,
	pub vlan_ip: String,
}

pub mod components;

pub async fn gen(
	server: &ServerCtx,
) -> GlobalResult<String> {
	let mut script = Vec::new();

	let mut prometheus_targets = HashMap::new();

	// MARK: Common (pre)
	script.push(components::common());
	script.push(components::node_exporter());
	script.push(components::sysctl());
	script.push(components::traefik());
	script.push(components::traefik_tunnel()?);

	prometheus_targets.insert(
		"node_exporter".into(),
		components::VectorPrometheusTarget {
			endpoint: "http://127.0.0.1:9100/metrics".into(),
			scrape_interval: 15,
		},
	);

	// MARK: Specific pool components
	match server.pool_type {
		backend::cluster::PoolType::Job => {
			script.push(components::docker());
			script.push(components::lz4());
			script.push(components::skopeo());
			script.push(components::umoci());
			script.push(components::cnitool());
			script.push(components::cni_plugins());
			script.push(components::nomad(server));
	
			prometheus_targets.insert(
				"nomad".into(),
				components::VectorPrometheusTarget {
					endpoint: "http://127.0.0.1:4646/v1/metrics?format=prometheus".into(),
					scrape_interval: 15,
				},
			);
		}
		backend::cluster::PoolType::Gg => {
			script.push(components::traefik_instance(components::TraefikInstance {
				name: "game_guard".into(),
				static_config: gg_traefik_static_config(server).await?,
				dynamic_config: String::new(),
				tls_certs: hashmap! {
					"letsencrypt_rivet_job".into() => components::TlsCert {
						cert_pem: env::var("TLS_CERT_LETSENCRYPT_RIVET_JOB_CERT_PEM")?,
						key_pem: env::var("TLS_CERT_LETSENCRYPT_RIVET_JOB_KEY_PEM")?,
					},
				},
				tcp_server_transports: Default::default(),
			}));
	
			prometheus_targets.insert(
				"game_guard".into(),
				components::VectorPrometheusTarget {
					endpoint: "http://127.0.0.1:9980/metrics".into(),
					scrape_interval: 15,
				},
			);
		}
		backend::cluster::PoolType::Ats => {
			script.push(components::docker());
			script.push(components::traffic_server(server).await?);
		}
	}

	// MARK: Common (post)
	if !prometheus_targets.is_empty() {
		script.push(components::vector(&components::VectorConfig {
			prometheus_targets,
		}));
	}

	let joined = script.join("\n\necho \"======\"\n\n");
	Ok(format!("#!/usr/bin/env bash\nset -eu\n\n{joined}"))
}

async fn gg_traefik_static_config(server: &ServerCtx) -> GlobalResult<String> {
	let api_route_token = &util::env::read_secret(&["rivet", "api_route", "token"]).await?;
	let http_provider_endpoint = format!(
		"http://127.0.0.1:{port}/traefik/config/game-guard?token={api_route_token}&region={region}",
		port = components::TUNNEL_API_ROUTE_PORT,
		region = server.universal_region_str
	);

	let mut config = formatdoc!(
		r#"
		[entryPoints]
			[entryPoints.traefik]
				address = ":9980"

			[entryPoints.lb-80]
				address = ":80"

			[entryPoints.lb-443]
				address = ":443"

		[api-routes]
			insecure = true

		[metrics.prometheus]
			# See lib/chirp/metrics/src/buckets.rs
			buckets = [0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0]
			addEntryPointsLabels = true
			addRoutersLabels = true
			addServicesLabels = true

		[providers]
			[providers.file]
				directory = "/etc/game_guard/dynamic"

			[providers.http]
				endpoint = "{http_provider_endpoint}"
				pollInterval = "0.5s"
		"#
	);

	// TCP ports
	for port in util::net::job::MIN_INGRESS_PORT_TCP..=util::net::job::MAX_INGRESS_PORT_TCP {
		config.push_str(&formatdoc!(
			r#"
			[entryPoints.lb-{port}-tcp]
				address = ":{port}/tcp"

			[entryPoints.lb-{port}-tcp.transport.respondingTimeouts]
				readTimeout = "12h"
				writeTimeout = "12h"
				idleTimeout = "30s"

			"#
		));
	}

	// UDP ports
	for port in util::net::job::MIN_INGRESS_PORT_UDP..=util::net::job::MAX_INGRESS_PORT_UDP {
		config.push_str(&formatdoc!(
			r#"
			[entryPoints.lb-{port}-udp]
				address = ":{port}/udp"

			[entryPoints.lb-{port}-udp.udp]
				timeout = "15s"
			"#
		));
	}

	Ok(config)
}
