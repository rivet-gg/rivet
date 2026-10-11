//! Namespace-scoped management of trusted external JWT signers.

use acl::types::TargetScope;
use anyhow::{Context, Result};
use axum::{
	http::Method,
	response::{IntoResponse, Json, Response},
};
use rivet_api_builder::{
	ApiBadRequest, ApiError, ApiNotFound,
	extract::{Extension, Path},
};
use rivet_api_peer::auth::issuers::{MutationRequest, MutationResponse, into_response};
use rivet_api_types::auth::issuers::{
	DeleteResponse, GetResponse, ListResponse, UpsertRequest, UpsertResponse,
};
use rivet_api_util::request_remote_datacenter;
use rivet_auth::{AccessRequest, Consistency, OperationKind, OwnedGrant, ResourceKind, Scope};
use rivet_auth_jwt::{
	ExternalIssuer, ExternalIssuerRegistry, ValidatedGrantSet, ops::external_issuers,
};
use serde::Deserialize;

use crate::ctx::ApiCtx;

#[derive(Deserialize)]
pub struct NamespacePath {
	pub namespace: String,
}

#[derive(Deserialize)]
pub struct IssuerPath {
	pub namespace: String,
	pub issuer_name: String,
}

#[utoipa::path(
	get, operation_id = "auth_issuers_list", path = "/namespaces/{namespace}/auth/issuers",
	params(("namespace" = String, Path, description = "Namespace name")),
	responses((status = 200, body = ListResponse)), security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip_all)]
pub async fn list(Extension(ctx): Extension<ApiCtx>, Path(path): Path<NamespacePath>) -> Response {
	match list_inner(ctx, path).await {
		Ok(result) => Json(result).into_response(),
		Err(error) => ApiError::from(error).into_response(),
	}
}

async fn list_inner(ctx: ApiCtx, path: NamespacePath) -> Result<ListResponse> {
	let namespace_id = resolve_namespace(&ctx, path.namespace).await?;
	let credential = ctx.authenticate(Consistency::Cached).await?;
	credential.require_acl()?.authorize(&AccessRequest {
		namespace: Scope::Id(namespace_id),
		resource: ResourceKind::Jwt,
		target: Scope::Any,
		operation: OperationKind::List,
	})?;
	let registry = ctx
		.op(external_issuers::get::Input { namespace_id })
		.await?
		.registry;
	Ok(ListResponse {
		issuers: registry.issuers.into_iter().map(into_response).collect(),
	})
}

#[utoipa::path(
	get, operation_id = "auth_issuers_get", path = "/namespaces/{namespace}/auth/issuers/{issuer_name}",
	params(("namespace" = String, Path, description = "Namespace name"), ("issuer_name" = String, Path, description = "Issuer registration name")),
	responses((status = 200, body = GetResponse)), security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip_all)]
pub async fn get(Extension(ctx): Extension<ApiCtx>, Path(path): Path<IssuerPath>) -> Response {
	match get_inner(ctx, path).await {
		Ok(result) => Json(result).into_response(),
		Err(error) => ApiError::from(error).into_response(),
	}
}

async fn get_inner(ctx: ApiCtx, path: IssuerPath) -> Result<GetResponse> {
	let namespace_id = resolve_namespace(&ctx, path.namespace).await?;
	let credential = ctx.authenticate(Consistency::Cached).await?;
	credential.require_acl()?.authorize(&AccessRequest {
		namespace: Scope::Id(namespace_id),
		resource: ResourceKind::Jwt,
		target: Scope::Any,
		operation: OperationKind::Read,
	})?;
	let registry = ctx
		.op(external_issuers::get::Input { namespace_id })
		.await?
		.registry;
	let issuer = registry
		.issuers
		.into_iter()
		.find(|entry| entry.name == path.issuer_name)
		.ok_or_else(|| ApiNotFound.build())?;
	Ok(GetResponse {
		issuer: into_response(issuer),
	})
}

#[utoipa::path(
	put, operation_id = "auth_issuers_upsert", path = "/namespaces/{namespace}/auth/issuers/{issuer_name}",
	params(("namespace" = String, Path, description = "Namespace name"), ("issuer_name" = String, Path, description = "Issuer registration name")),
	request_body(content = UpsertRequest, content_type = "application/json"),
	responses((status = 200, body = UpsertResponse)), security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip_all)]
pub async fn upsert(
	Extension(ctx): Extension<ApiCtx>,
	Path(path): Path<IssuerPath>,
	rivet_api_builder::extract::Json(body): rivet_api_builder::extract::Json<UpsertRequest>,
) -> Response {
	match upsert_inner(ctx, path, body).await {
		Ok(result) => Json(result).into_response(),
		Err(error) => ApiError::from(error).into_response(),
	}
}

async fn upsert_inner(
	ctx: ApiCtx,
	path: IssuerPath,
	body: UpsertRequest,
) -> Result<UpsertResponse> {
	let namespace_id = resolve_namespace(&ctx, path.namespace).await?;
	let credential = ctx.authenticate(Consistency::Authoritative).await?;
	let acl = credential.require_acl()?;
	acl.authorize(&AccessRequest {
		namespace: Scope::Id(namespace_id),
		resource: ResourceKind::Jwt,
		target: Scope::Any,
		operation: OperationKind::Update,
	})?;
	let grants = ValidatedGrantSet::new(
		namespace_id,
		body.grant_ceiling.iter().cloned().map(|grant| OwnedGrant {
			namespace: Scope::Id(namespace_id),
			resource: grant.resource,
			target: match grant.target {
				TargetScope::Any => Scope::Any,
				TargetScope::Id(id) => Scope::Id(id),
			},
			operations: grant.operations,
		}),
	)
	.map_err(|_| {
		ApiBadRequest {
			reason: "invalid issuer grant ceiling".into(),
		}
		.build()
	})?;
	if !rivet_auth::can_delegate(acl.authority().grants(), grants.grants()) {
		return Err(rivet_auth::errors::Auth::InsufficientPermissions.build());
	}
	let entry = ExternalIssuer {
		name: path.issuer_name.clone(),
		issuer: body.issuer.clone(),
		audience: body.audience.clone(),
		jwks_url: body.jwks_url.clone(),
		algorithm: body.algorithm.clone(),
		grant_ceiling: grants.into_grants(),
	};
	ExternalIssuerRegistry {
		namespace_id,
		generation: 1,
		issuers: vec![entry],
	}
	.validate()
	.map_err(|_| {
		ApiBadRequest {
			reason: "invalid issuer registration".into(),
		}
		.build()
	})?;
	let response = mutate(
		&ctx,
		MutationRequest {
			namespace_id,
			name: path.issuer_name,
			replacement: Some(body),
		},
	)
	.await?;
	Ok(UpsertResponse {
		issuer: response.issuer.context("missing registered issuer")?,
	})
}

#[utoipa::path(
	delete, operation_id = "auth_issuers_delete", path = "/namespaces/{namespace}/auth/issuers/{issuer_name}",
	params(("namespace" = String, Path, description = "Namespace name"), ("issuer_name" = String, Path, description = "Issuer registration name")),
	responses((status = 200, body = DeleteResponse)), security(("bearer_auth" = [])),
)]
#[tracing::instrument(skip_all)]
pub async fn delete(Extension(ctx): Extension<ApiCtx>, Path(path): Path<IssuerPath>) -> Response {
	match delete_inner(ctx, path).await {
		Ok(result) => Json(result).into_response(),
		Err(error) => ApiError::from(error).into_response(),
	}
}

async fn delete_inner(ctx: ApiCtx, path: IssuerPath) -> Result<DeleteResponse> {
	let namespace_id = resolve_namespace(&ctx, path.namespace).await?;
	// Like registration, removal must not rely on a stale ACL grant or a JWT.
	let credential = ctx.authenticate(Consistency::Authoritative).await?;
	credential.require_acl()?.authorize(&AccessRequest {
		namespace: Scope::Id(namespace_id),
		resource: ResourceKind::Jwt,
		target: Scope::Any,
		operation: OperationKind::Delete,
	})?;
	mutate(
		&ctx,
		MutationRequest {
			namespace_id,
			name: path.issuer_name,
			replacement: None,
		},
	)
	.await?;
	Ok(DeleteResponse {})
}

async fn mutate(ctx: &ApiCtx, request: MutationRequest) -> Result<MutationResponse> {
	if ctx.config().is_leader() {
		rivet_api_peer::auth::issuers::mutate(ctx.clone().into(), (), (), request).await
	} else {
		let leader_dc = ctx.config().leader_dc()?;
		request_remote_datacenter::<MutationResponse>(
			ctx.config(),
			leader_dc.datacenter_label,
			"/auth/issuers/mutate",
			Method::POST,
			Option::<&()>::None,
			Some(&request),
		)
		.await
	}
}

async fn resolve_namespace(ctx: &ApiCtx, name: String) -> Result<rivet_util::Id> {
	Ok(ctx
		.op(namespace::ops::resolve_for_name_global::Input { name })
		.await?
		.ok_or_else(|| namespace::errors::Namespace::NotFound.build())?
		.namespace_id)
}
