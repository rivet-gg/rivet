use std::collections::HashMap;

use chirp_workflow::prelude::*;
use fdb_util::{FormalKey, SERIALIZABLE, SNAPSHOT};
use foundationdb::{self as fdb, options::StreamingMode};
use futures_util::TryStreamExt;

use crate::keys;

#[derive(Debug, Default)]
pub struct Input {
	pub server_id: Uuid,
	pub protocols: Vec<protocol::GameGuardProtocol>,
}

#[derive(Debug)]
pub struct Output {
	pub ports: Vec<Uuid>,
}

#[operation]
pub async fn ds_server_allocate_ingress_port(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	/// Choose which port to assign for a job's ingress port.
	/// This is required because TCP and UDP do not have a `Host` header and thus cannot be re-routed by hostname.
	///
	/// If not provided by `ProxiedPort`, then:
	/// - HTTP: 80
	/// - HTTPS: 443
	/// - TCP/TLS: random
	/// - UDP: random

	let gg_config = &ctx.config().server()?.rivet.guard;

	match protocol {
		GameGuardProtocol::Http => Ok(gg_config.http_port()),
		GameGuardProtocol::Https => Ok(gg_config.https_port()),
		GameGuardProtocol::Tcp | GameGuardProtocol::TcpTls => {
			bind_with_retries(
				ctx,
				tx,
				protocol,
				gg_config.min_ingress_port_tcp()..=gg_config.max_ingress_port_tcp(),
			)
			.await
		}
		GameGuardProtocol::Udp => {
			bind_with_retries(
				ctx,
				tx,
				protocol,
				gg_config.min_ingress_port_udp()..=gg_config.max_ingress_port_udp(),
			)
			.await
		}
	}
}
