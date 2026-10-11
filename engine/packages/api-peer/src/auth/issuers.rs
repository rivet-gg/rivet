//! Leader-local registration mutations. The public API checks the caller before forwarding here.

use anyhow::{Context, Result};
use gas::prelude::*;
use rivet_api_builder::ApiCtx;
use rivet_api_types::auth::issuers::{Grant, Issuer, UpsertRequest};
use rivet_auth_jwt::{ExternalIssuer, ValidatedGrantSet, ops::external_issuers::mutate};
use rivet_auth_policy::{OwnedGrant, Scope};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutationRequest {
	pub namespace_id: Id,
	pub name: String,
	pub replacement: Option<UpsertRequest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MutationResponse {
	pub issuer: Option<Issuer>,
}

pub async fn mutate(
	ctx: ApiCtx,
	_path: (),
	_query: (),
	request: MutationRequest,
) -> Result<MutationResponse> {
	let replacement = request
		.replacement
		.map(|entry| {
			let grants = ValidatedGrantSet::new(
				request.namespace_id,
				entry.grant_ceiling.into_iter().map(|grant| OwnedGrant {
					namespace: Scope::Id(request.namespace_id),
					resource: grant.resource,
					target: match grant.target {
						acl::types::TargetScope::Any => Scope::Any,
						acl::types::TargetScope::Id(id) => Scope::Id(id),
					},
					operations: grant.operations,
				}),
			)?;
			Ok::<_, anyhow::Error>(ExternalIssuer {
				name: request.name.clone(),
				issuer: entry.issuer,
				audience: entry.audience,
				jwks_url: entry.jwks_url,
				algorithm: entry.algorithm,
				grant_ceiling: grants.into_grants(),
			})
		})
		.transpose()?;
	let registry = ctx
		.op(mutate::Input {
			namespace_id: request.namespace_id,
			name: request.name.clone(),
			replacement: replacement.clone(),
		})
		.await?;
	tracing::info!(
		namespace_id = %request.namespace_id,
		issuer_name = %request.name,
		generation = registry.generation,
		action = if replacement.is_some() { "upsert" } else { "delete" },
		"JWT issuer registration mutation"
	);
	let issuer = registry
		.issuers
		.into_iter()
		.find(|entry| entry.name == request.name)
		.map(into_response);
	if replacement.is_some() {
		Context::context(issuer.as_ref(), "issuer registration missing after commit")?;
	}
	Ok(MutationResponse { issuer })
}

pub fn into_response(entry: ExternalIssuer) -> Issuer {
	Issuer {
		name: entry.name,
		issuer: entry.issuer,
		audience: entry.audience,
		jwks_url: entry.jwks_url,
		algorithm: entry.algorithm,
		grant_ceiling: entry
			.grant_ceiling
			.into_iter()
			.map(|grant| Grant {
				resource: grant.resource,
				target: match grant.target {
					Scope::Any => acl::types::TargetScope::Any,
					Scope::Id(id) => acl::types::TargetScope::Id(id),
				},
				operations: grant.operations,
			})
			.collect(),
	}
}
