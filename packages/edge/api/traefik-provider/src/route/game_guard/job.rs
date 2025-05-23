use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
};

use api_core_traefik_provider::types;
use api_helper::ctx::Ctx;
use proto::backend::{self, pkg::*};
use redis::AsyncCommands;
use rivet_operation::prelude::*;
use url::Url;

use crate::auth::Auth;

/// Builds configuration for job routes.
#[tracing::instrument(skip(ctx))]
pub async fn build_job(
	ctx: &Ctx<Auth>,
	config: &mut types::TraefikConfigResponse,
) -> GlobalResult<()> {
	let dc_id = ctx.config().server()?.rivet.edge()?.datacenter_id;
	let redis_job = ctx.op_ctx().redis_job().await?;
	let job_runs_fetch = fetch_job_runs(redis_job, dc_id).await?;

	config.http.middlewares.insert(
		"job-rate-limit".to_owned(),
		types::TraefikMiddlewareHttp::RateLimit {
			average: 100,
			period: "5m".into(),
			burst: 256,
			source_criterion: types::InFlightReqSourceCriterion::IpStrategy(types::IpStrategy {
				depth: 0,
				exclude_ips: None,
			}),
		},
	);
	config.http.middlewares.insert(
		"job-in-flight".to_owned(),
		types::TraefikMiddlewareHttp::InFlightReq {
			// This number needs to be high to allow for parallel requests
			amount: 4,
			source_criterion: types::InFlightReqSourceCriterion::IpStrategy(types::IpStrategy {
				depth: 0,
				exclude_ips: None,
			}),
		},
	);

	// Process proxied ports
	for run_proxied_ports in &job_runs_fetch {
		let run_id = unwrap_ref!(run_proxied_ports.run_id);
		tracing::debug!(proxied_ports_len = ?run_proxied_ports.proxied_ports.len(), "adding job run");
		for proxied_port in &run_proxied_ports.proxied_ports {
			let register_res = job_register_proxied_port(**run_id, proxied_port, config);
			match register_res {
				Ok(_) => {}
				Err(err) => {
					tracing::error!(?err, ?proxied_port, "failed to register proxied port route")
				}
			}
		}
	}

	Ok(())
}

#[tracing::instrument(skip(redis_job))]
async fn fetch_job_runs(
	mut redis_job: RedisPool,
	region_id: Uuid,
) -> GlobalResult<Vec<job::redis_job::RunProxiedPorts>> {
	let runs = redis_job
		.hvals::<_, Vec<Vec<u8>>>(util_job::key::proxied_ports(region_id))
		.await?
		.into_iter()
		.filter_map(
			|buf| match job::redis_job::RunProxiedPorts::decode(buf.as_slice()) {
				Ok(x) => Some(x),
				Err(err) => {
					tracing::error!(?err, "failed to decode run RunProxiedPorts from redis");
					None
				}
			},
		)
		.collect::<Vec<_>>();
	let proxied_port_len = runs.iter().fold(0, |acc, x| acc + x.proxied_ports.len());
	tracing::debug!(runs_len = ?runs.len(), ?proxied_port_len, "fetched job runs");
	Ok(runs)
}

#[tracing::instrument(skip(config))]
fn job_register_proxied_port(
	run_id: Uuid,
	proxied_port: &job::redis_job::run_proxied_ports::ProxiedPort,
	config: &mut types::TraefikConfigResponse,
) -> GlobalResult<()> {
	use backend::job::ProxyProtocol;

	let ingress_port = proxied_port.ingress_port;
	let target_nomad_port_label = unwrap_ref!(proxied_port.target_nomad_port_label);
	let service_id = format!("job-run:{}:{}", run_id, target_nomad_port_label);
	let proxy_protocol = unwrap!(backend::job::ProxyProtocol::from_i32(
		proxied_port.proxy_protocol
	));

	// Insert the relevant service
	match proxy_protocol {
		ProxyProtocol::Http | ProxyProtocol::Https => {
			config.http.services.insert(
				service_id.clone(),
				types::TraefikService {
					load_balancer: types::TraefikLoadBalancer {
						servers: vec![types::TraefikServer {
							url: Some(format!(
								"http://{}:{}",
								proxied_port.ip, proxied_port.source
							)),
							address: None,
						}],
						sticky: None,
					},
				},
			);
		}
		ProxyProtocol::Tcp | ProxyProtocol::TcpTls => {
			config.tcp.services.insert(
				service_id.clone(),
				types::TraefikService {
					load_balancer: types::TraefikLoadBalancer {
						servers: vec![types::TraefikServer {
							url: None,
							address: Some(format!("{}:{}", proxied_port.ip, proxied_port.source)),
						}],
						sticky: None,
					},
				},
			);
		}
		ProxyProtocol::Udp => {
			config.udp.services.insert(
				service_id.clone(),
				types::TraefikService {
					load_balancer: types::TraefikLoadBalancer {
						servers: vec![types::TraefikServer {
							url: None,
							address: Some(format!("{}:{}", proxied_port.ip, proxied_port.source)),
						}],
						sticky: None,
					},
				},
			);
		}
	};

	// Insert the relevant router
	match proxy_protocol {
		ProxyProtocol::Http => {
			// Generate config
			let middlewares =
				http_router_middlewares(run_id, proxied_port, target_nomad_port_label, config);
			let rule = format_http_rule(proxied_port);

			// Hash key
			let unique_key = (&run_id, &target_nomad_port_label, &rule, &middlewares);
			let mut hasher = DefaultHasher::new();
			unique_key.hash(&mut hasher);
			let hash = hasher.finish();

			config.http.routers.insert(
				format!("job-run:{run_id}:{hash:x}:http"),
				types::TraefikRouter {
					entry_points: vec![format!("lb-{ingress_port}")],
					rule: Some(rule),
					priority: None,
					service: service_id.clone(),
					middlewares,
					tls: None,
				},
			);
		}
		ProxyProtocol::Https => {
			// Generate config
			let middlewares =
				http_router_middlewares(run_id, proxied_port, target_nomad_port_label, config);
			let rule = format_http_rule(proxied_port);

			// Hash key
			let unique_key = (&run_id, &target_nomad_port_label, &rule, &middlewares);
			let mut hasher = DefaultHasher::new();
			unique_key.hash(&mut hasher);
			let hash = hasher.finish();

			config.http.routers.insert(
				format!("job-run:{run_id}:{hash:x}:https"),
				types::TraefikRouter {
					entry_points: vec![format!("lb-{ingress_port}")],
					rule: Some(rule),
					priority: None,
					service: service_id.clone(),
					middlewares,
					tls: Some(types::TraefikTls::build(build_tls_domains(proxied_port)?)),
				},
			);
		}
		ProxyProtocol::Tcp => {
			config.tcp.routers.insert(
				format!("job-run:{}:{}:tcp", run_id, target_nomad_port_label),
				types::TraefikRouter {
					entry_points: vec![format!("lb-{ingress_port}-tcp")],
					rule: Some("HostSNI(`*`)".into()),
					priority: None,
					service: service_id,
					middlewares: vec![],
					tls: None,
				},
			);
		}
		ProxyProtocol::TcpTls => {
			config.tcp.routers.insert(
				format!("job-run:{}:{}:tcp-tls", run_id, target_nomad_port_label),
				types::TraefikRouter {
					entry_points: vec![format!("lb-{ingress_port}-tcp")],
					rule: Some("HostSNI(`*`)".into()),
					priority: None,
					service: service_id,
					middlewares: vec![],
					tls: Some(types::TraefikTls::build(build_tls_domains(proxied_port)?)),
				},
			);
		}
		ProxyProtocol::Udp => {
			config.udp.routers.insert(
				format!("job-run:{}:{}:udp", run_id, target_nomad_port_label),
				types::TraefikRouter {
					entry_points: vec![format!("lb-{ingress_port}-udp")],
					rule: None,
					priority: None,
					service: service_id,
					middlewares: vec![],
					tls: None,
				},
			);
		}
	}

	Ok(())
}

fn format_http_rule(proxied_port: &job::redis_job::run_proxied_ports::ProxiedPort) -> String {
	proxied_port
		.ingress_hostnames
		.iter()
		.map(|x| {
			if let Ok(url) = Url::parse(&format!("https://{x}")) {
				if let (true, Some(host)) = (url.path() != "/", url.host()) {
					return format!("(Host(`{host}`) && PathPrefix(`{}`))", url.path());
				}
			}

			format!("Host(`{x}`)")
		})
		.collect::<Vec<String>>()
		.join(" || ")
}

fn build_tls_domains(
	proxied_port: &job::redis_job::run_proxied_ports::ProxiedPort,
) -> GlobalResult<Vec<types::TraefikTlsDomain>> {
	// Derive TLS config. Jobs can specify their own ingress rules, so we
	// need to derive which domains to use for the job.
	//
	// An exact SSL mode will only work with one specific domain. This is
	// very rarely used.
	//
	// A parent wildcard SSL mode will use the parent domain as the SSL
	// name.
	let ssl_domain_mode = unwrap!(backend::job::SslDomainMode::from_i32(
		proxied_port.ssl_domain_mode,
	));
	let mut domains = Vec::new();
	match ssl_domain_mode {
		backend::job::SslDomainMode::Exact => {
			for host in &proxied_port.ingress_hostnames {
				domains.push(types::TraefikTlsDomain {
					main: host.clone(),
					sans: Vec::new(),
				});
			}
		}
		backend::job::SslDomainMode::ParentWildcard => {
			for host in &proxied_port.ingress_hostnames {
				let (_, parent_host) = unwrap!(host.split_once('.'));
				domains.push(types::TraefikTlsDomain {
					main: parent_host.to_owned(),
					sans: vec![format!("*.{}", parent_host)],
				});
			}
		}
	}

	Ok(domains)
}

fn http_router_middlewares(
	run_id: Uuid,
	proxied_port: &job::redis_job::run_proxied_ports::ProxiedPort,
	target_nomad_port_label: &str,
	config: &mut types::TraefikConfigResponse,
) -> Vec<String> {
	let mut middlewares = vec!["job-rate-limit".to_string(), "job-in-flight".to_string()];

	// Check if any of the hostname values have paths
	let paths = proxied_port
		.ingress_hostnames
		.iter()
		.flat_map(|url| Url::parse(&format!("https://{url}")))
		.filter(|url| url.path() != "/");

	// Hash key
	let unique_key = (
		&run_id,
		&target_nomad_port_label,
		&proxied_port.ingress_hostnames,
	);
	let mut hasher = DefaultHasher::new();
	unique_key.hash(&mut hasher);
	let hash = hasher.finish();

	// Create strip prefix middleware
	if paths.clone().count() != 0 {
		let strip_prefix_id = format!("job-run-strip-prefix:{run_id}:{hash:x}");

		config.http.middlewares.insert(
			strip_prefix_id.clone(),
			types::TraefikMiddlewareHttp::StripPrefix {
				prefixes: paths.map(|url| url.path().to_string()).collect(),
			},
		);

		middlewares.push(strip_prefix_id);
	}

	middlewares
}
