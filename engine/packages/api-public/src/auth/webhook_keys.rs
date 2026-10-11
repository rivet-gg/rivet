use super::signing;
use crate::{ctx::ApiCtx, errors::Signing};
use anyhow::Result;
use axum::response::{IntoResponse, Response};
use rivet_api_builder::extract::{Extension, Json, Path};
use rivet_api_peer::auth::webhooks::MutationRequest;
use rivet_api_types::auth::webhooks::{KeyStatus, Metadata, RevokeRequest, RotateRequest};
use rivet_auth::{AccessRequest, Consistency, OperationKind, ResourceKind, Scope};
use rivet_auth_jwt::ops::webhook::Change;
use rivet_util::Id;

#[utoipa::path(post, operation_id = "auth_webhooks_ensure", path = "/auth/webhooks/ensure",
    responses((status = 200, body = Metadata)), security(("bearer_auth" = [])))]
#[tracing::instrument(skip_all)]
pub async fn ensure(Extension(ctx): Extension<ApiCtx>) -> Response {
	let result: Result<Metadata> = async {
		let (namespace_id, _) = signing::credential(&ctx, ResourceKind::WebhookSigning).await?;
		signing::charge(
			&ctx,
			namespace_id,
			rivet_auth_jwt::ops::signing_quota::Charge::Webhook,
		)
		.await?;
		mutate(&ctx, namespace_id, Change::Ensure).await?;
		super::webhook_discovery::get(&ctx, namespace_id)
			.await?
			.metadata
			.ok_or_else(|| Signing::Unavailable.build())
	}
	.await;
	signing::no_store(match result {
		Ok(body) => Json(body).into_response(),
		Err(error) => signing::error_response(error),
	})
}

#[utoipa::path(get, operation_id = "auth_webhook_keys_get", path = "/namespaces/{namespace}/auth/webhook-keys",
    params(("namespace" = String, Path, description = "Namespace name")),
    responses((status = 200, body = KeyStatus)), security(("bearer_auth" = [])))]
#[tracing::instrument(skip_all)]
pub async fn get(Extension(ctx): Extension<ApiCtx>, Path((name,)): Path<(String,)>) -> Response {
	respond(
		async {
			let (id, _) = authorize(&ctx, name, OperationKind::Read).await?;
			if ctx.config().is_leader() {
				rivet_api_peer::auth::webhooks::get(ctx.into(), (id,), ()).await
			} else {
				rivet_api_util::request_remote_datacenter(
					ctx.config(),
					ctx.config().leader_dc()?.datacenter_label,
					&format!("/auth/webhooks/{id}/status"),
					rivet_api_util::Method::GET,
					Option::<&()>::None,
					Option::<&()>::None,
				)
				.await
			}
		}
		.await,
	)
}

#[utoipa::path(post, operation_id = "auth_webhook_keys_rotate", path = "/namespaces/{namespace}/auth/webhook-keys/rotate",
    params(("namespace" = String, Path, description = "Namespace name")), request_body = RotateRequest,
    responses((status = 200, body = KeyStatus)), security(("bearer_auth" = [])))]
#[tracing::instrument(skip_all)]
pub async fn rotate(
	Extension(ctx): Extension<ApiCtx>,
	Path((name,)): Path<(String,)>,
	request: axum::extract::Request,
) -> Response {
	respond(
		async {
			let (id, _) = authorize(&ctx, name.clone(), OperationKind::Update).await?;
			let body: RotateRequest = signing::management_body(request).await?;
			let (current_id, deadline) = authorize(&ctx, name, OperationKind::Update).await?;
			if current_id != id {
				return Err(rivet_auth::errors::Auth::InsufficientPermissions.build());
			}
			mutate_admitted(
				&ctx,
				id,
				Change::Rotate {
					expected_generation: body.expected_generation,
				},
				Some(deadline),
			)
			.await
		}
		.await,
	)
}

#[utoipa::path(post, operation_id = "auth_webhook_keys_revoke", path = "/namespaces/{namespace}/auth/webhook-keys/revoke",
    params(("namespace" = String, Path, description = "Namespace name")), request_body = RevokeRequest,
    responses((status = 200, body = KeyStatus)), security(("bearer_auth" = [])))]
#[tracing::instrument(skip_all)]
pub async fn revoke(
	Extension(ctx): Extension<ApiCtx>,
	Path((name,)): Path<(String,)>,
	request: axum::extract::Request,
) -> Response {
	respond(
		async {
			let (id, _) = authorize(&ctx, name.clone(), OperationKind::Delete).await?;
			let body: RevokeRequest = signing::management_body(request).await?;
			let (current_id, deadline) = authorize(&ctx, name, OperationKind::Delete).await?;
			if current_id != id {
				return Err(rivet_auth::errors::Auth::InsufficientPermissions.build());
			}
			let request_id = gas::prelude::Uuid::parse_str(&body.request_id)
				.map_err(|_| Signing::InvalidInput.build())?
				.into_bytes();
			mutate_admitted(
				&ctx,
				id,
				Change::Revoke {
					request_id,
					expected_generation: body.expected_generation,
					kids: body.kids,
				},
				Some(deadline),
			)
			.await
		}
		.await,
	)
}

async fn authorize(ctx: &ApiCtx, name: String, operation: OperationKind) -> Result<(Id, i64)> {
	let started = rivet_util::timestamp::now();
	let credential = ctx.authenticate(Consistency::Authoritative).await?;
	let acl = credential.require_acl()?;
	ctx.signing_audit.caller(acl.token_id());
	let ns = ctx
		.op(namespace::ops::resolve_for_name_global::Input { name })
		.await?
		.ok_or_else(|| Signing::NotFound.build())?;
	acl.authorize(&AccessRequest {
		namespace: Scope::Id(ns.namespace_id),
		resource: ResourceKind::WebhookSigning,
		target: Scope::Any,
		operation,
	})?;
	signing::namespace_exists(ctx, ns.namespace_id).await?;
	if acl
		.expires_ts()
		.is_some_and(|exp| exp <= rivet_util::timestamp::now())
	{
		return Err(rivet_auth::errors::Auth::TokenExpired.build());
	}
	let deadline = admission_deadline(started, acl.expires_ts());
	rivet_auth_jwt::ops::webhook::mutate::check_admission(
		Some(deadline),
		rivet_util::timestamp::now(),
	)
	.map_err(|_| Signing::Unavailable.build())?;
	ctx.signing_audit.identity(acl.token_id(), ns.namespace_id);
	Ok((ns.namespace_id, deadline))
}

fn admission_deadline(started: i64, expires_ts: Option<i64>) -> i64 {
	expires_ts
		.unwrap_or(i64::MAX)
		.min(started.saturating_add(5_000))
}

pub(crate) async fn mutate(ctx: &ApiCtx, namespace_id: Id, change: Change) -> Result<KeyStatus> {
	mutate_admitted(ctx, namespace_id, change, None).await
}

async fn mutate_admitted(
	ctx: &ApiCtx,
	namespace_id: Id,
	change: Change,
	admission_deadline_ts: Option<i64>,
) -> Result<KeyStatus> {
	let request = MutationRequest {
		namespace_id,
		change,
		admission_deadline_ts,
	};
	let status = if ctx.config().is_leader() {
		rivet_api_peer::auth::webhooks::mutate(ctx.clone().into(), (), (), request).await
	} else {
		rivet_api_util::request_remote_datacenter(
			ctx.config(),
			ctx.config().leader_dc()?.datacenter_label,
			"/auth/webhooks/mutate",
			rivet_api_util::Method::POST,
			Option::<&()>::None,
			Some(&request),
		)
		.await
	}?;
	ctx.signing_audit
		.key(status.active_kid.clone(), status.generation);
	Ok(status)
}

fn respond(result: Result<KeyStatus>) -> Response {
	signing::no_store(match result {
		Ok(body) => Json(body).into_response(),
		Err(error) => signing::error_response(error),
	})
}

#[cfg(test)]
#[path = "../../tests/unit/webhook_management.rs"]
mod tests;
