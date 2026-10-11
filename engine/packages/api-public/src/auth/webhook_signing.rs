use super::{signing, webhook_keys};
use crate::{ctx::ApiCtx, errors::Signing};
use anyhow::Result;
use axum::{
	extract::Request,
	response::{IntoResponse, Response},
};
use rivet_api_builder::extract::{Extension, Json};
use rivet_api_peer::auth::webhook_signing::{self as peer, DEADLINE_HEADER};
use rivet_api_types::auth::webhooks::{SignResponse, WebhookPayload};
use rivet_auth::ResourceKind;

#[utoipa::path(post, operation_id = "auth_webhooks_sign", path = "/auth/webhooks/sign",
    request_body(content = inline(WebhookPayload), content_type = "application/octet-stream"),
    responses((status = 200, body = SignResponse)), security(("bearer_auth" = [])))]
#[tracing::instrument(skip_all)]
pub async fn sign(Extension(ctx): Extension<ApiCtx>, request: Request) -> Response {
	signing::no_store(match sign_inner(&ctx, request).await {
		Ok(body) => Json(body).into_response(),
		Err(error) => signing::error_response(error),
	})
}

async fn sign_inner(ctx: &ApiCtx, request: Request) -> Result<SignResponse> {
	let (namespace_id, _) = signing::credential(ctx, ResourceKind::WebhookSigning).await?;
	signing::charge(
		ctx,
		namespace_id,
		rivet_auth_jwt::ops::signing_quota::Charge::Webhook,
	)
	.await?;
	// Provisions lazily and repairs workflow dispatch after an interrupted ensure.
	webhook_keys::mutate(
		ctx,
		namespace_id,
		rivet_auth_jwt::ops::webhook::Change::Ensure,
	)
	.await?;
	let cap = ctx.config().auth_required()?.jwt.signing.payload_bytes as usize;
	let (body, permit) = peer::read_body_observed(
		request,
		cap,
		|bytes| {
			signing::charge(
				ctx,
				namespace_id,
				rivet_auth_jwt::ops::signing_quota::Charge::Bytes(bytes),
			)
		},
		|bytes| ctx.signing_audit.bytes(bytes),
	)
	.await?;
	ctx.signing_audit.bytes(body.len() as u64);
	let check_started = rivet_util::timestamp::now();
	let (current_namespace, acl) = signing::credential(ctx, ResourceKind::WebhookSigning).await?;
	if current_namespace != namespace_id {
		return Err(rivet_auth::errors::Auth::InsufficientPermissions.build());
	}
	let deadline = acl
		.expires_ts()
		.unwrap_or(i64::MAX)
		.min(check_started.saturating_add(5_000));
	if ctx.config().is_leader() {
		let signed = peer::sign_body(ctx, namespace_id, deadline, body, permit).await?;
		ctx.signing_audit
			.key(signed.response.kid.clone(), signed.generation);
		return Ok(signed.response);
	}
	let _permit = permit;
	let url = ctx
		.config()
		.leader_dc()?
		.peer_url
		.join(&format!("/auth/webhooks/{namespace_id}/sign"))?;
	let response = rivet_pools::reqwest::client()
		.await?
		.post(url)
		.header("content-type", "application/octet-stream")
		.header(DEADLINE_HEADER, deadline.to_string())
		.timeout(std::time::Duration::from_secs(5))
		.body(body)
		.send()
		.await
		.map_err(|_| Signing::Unavailable.build())?;
	let result: peer::Signed = rivet_api_util::parse_response(response).await?;
	if rivet_util::timestamp::now() >= deadline {
		return Err(Signing::Unavailable.build());
	}
	ctx.signing_audit
		.key(result.response.kid.clone(), result.generation);
	Ok(result.response)
}
