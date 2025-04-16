use chirp_workflow::prelude::*;
use fdb_util::FormalKey; // Added for deserialize method
use global_error::GlobalResult;
use rivet_guard_core::proxy_service::{RouteTarget, RoutingResult, RoutingTimeout};
use std::collections::HashMap;
use uuid::Uuid;

/// Route requests to actors based on the route configuration
pub async fn route_via_route_config(
	ctx: &StandaloneCtx,
	host: &str,
	path: &str,
	dc_id: Uuid,
) -> GlobalResult<Option<RoutingResult>> {
	// Get route directly using hostname and path
	// The operation handles priority internally and returns the best match
	let routes_res = ctx
		.op(route::ops::get_by_hostname_path::Input {
			hostname: host.to_string(),
			path: path.to_string(),
		})
		.await?;

	// If we didn't find any route, return None
	let Some(route) = routes_res.route else {
		return Ok(None);
	};

	let namespace_id = route.namespace_id;

	tracing::debug!(
		host = host,
		path = path,
		route_id = %route.route_id,
		"Found matching route"
	);

	// Convert the selector tags from the route
	let selector_tags: HashMap<String, String> = route.selector_tags.clone();

	// Query actors with matching tags in this environment
	let actors_res = ctx
		.op(pegboard::ops::actor::list_for_env::Input {
			env_id: namespace_id,
			tags: selector_tags,
			include_destroyed: false,
			created_before: None,
			limit: 50, // Reasonable limit for load balancing
		})
		.await?;

	if actors_res.actors.is_empty() {
		tracing::warn!(
			host = host,
			path = path,
			route_id = %route.route_id,
			"Found matching route but no actors with matching tags"
		);
		return Ok(None);
	}

	// Fetch each actor's details to get their connection information
	let mut targets = Vec::new();

	for actor_entry in &actors_res.actors {
		// Find actor's proxied ports
		if let Some(actor_targets) = find_actor_targets(
			ctx,
			&actor_entry.actor_id,
			dc_id,
			// Forward the path, accounting for route_subpaths
			if route.route_subpaths && path.len() > route.path.len() {
				&path[route.path.len()..]
			} else {
				"/"
			},
		)
		.await?
		{
			targets.extend(actor_targets);
		}
	}

	if targets.is_empty() {
		tracing::warn!(
			host = host,
			path = path,
			route_id = %route.route_id,
			"Found matching actors but no valid targets"
		);
		return Ok(None);
	}

	// Return routing result with all targets for load balancing
	Ok(Some(RoutingResult {
		targets,
		timeout: RoutingTimeout {
			routing_timeout: 10, // 10 seconds timeout
		},
	}))
}

/// Find all potential targets for an actor
async fn find_actor_targets(
	ctx: &StandaloneCtx,
	actor_id: &Uuid,
	_dc_id: Uuid, // Unused but kept for API compatibility
	path_to_forward: &str,
) -> GlobalResult<Option<Vec<RouteTarget>>> {
	// Fetch the actor's ports
	let proxied_ports = ctx
		.fdb()
		.await?
		.run(|tx, _mc| async move {
			// NOTE: This is not SERIALIZABLE because we don't want to conflict with port updates
			// and its not important if its slightly stale
			let proxied_ports_key = pegboard::keys::actor::ProxiedPortsKey::new(*actor_id);
			let raw = tx
				.get(
					&pegboard::keys::subspace().pack(&proxied_ports_key),
					fdb_util::SNAPSHOT,
				)
				.await?;
			if let Some(raw) = raw {
				let proxied_ports = proxied_ports_key
					.deserialize(&raw)
					.map_err(|x| foundationdb::FdbBindingError::CustomError(x.into()))?;
				Ok(Some(proxied_ports))
			} else {
				Ok(None)
			}
		})
		.await?;

	let Some(proxied_ports) = proxied_ports else {
		// Actor exists but has no proxied ports
		return Ok(None);
	};

	// Create targets for each proxied port
	let mut targets = Vec::new();
	for pp in proxied_ports {
		targets.push(RouteTarget {
			actor_id: Some(*actor_id),
			server_id: None,
			host: pp.lan_hostname.parse()?,
			port: pp.source,
			path: path_to_forward.to_string(),
		});
	}

	Ok(Some(targets))
}
