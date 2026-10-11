use crate::{ctx::ApiCtx, errors::Signing};
use anyhow::Result;
use rivet_auth::{Consistency, ResourceKind};
use rivet_util::Id;

pub async fn credential(
	ctx: &ApiCtx,
	resource: ResourceKind,
) -> Result<(Id, rivet_auth::AclCredential)> {
	if !ctx.config().auth_required()?.jwt.issuance_enabled()
		|| ctx.config().protocols().acl_rule.version() < 3
	{
		ctx.skip_auth();
		return Err(Signing::Unavailable.build());
	}
	let authenticated = ctx.authenticate(Consistency::Authoritative).await?;
	let acl = authenticated.require_acl()?;
	ctx.signing_audit.caller(acl.token_id());
	let namespace_id = acl.signing_namespace(resource)?;
	namespace_exists(ctx, namespace_id).await?;
	if acl
		.expires_ts()
		.is_some_and(|exp| exp <= rivet_util::timestamp::now())
	{
		return Err(rivet_auth::errors::Auth::TokenExpired.build());
	}
	ctx.signing_audit.identity(acl.token_id(), namespace_id);
	Ok((namespace_id, acl.clone()))
}

pub async fn namespace_exists(ctx: &ApiCtx, namespace_id: Id) -> Result<()> {
	let exists = if ctx.config().is_leader() {
		ctx.op(namespace::ops::exists_authoritative::Input { namespace_id })
			.await?
	} else {
		rivet_api_util::request_remote_datacenter::<bool>(
			ctx.config(),
			ctx.config().leader_dc()?.datacenter_label,
			&format!("/namespaces/{namespace_id}/exists-authoritative"),
			rivet_api_util::Method::GET,
			Option::<&()>::None,
			Option::<&()>::None,
		)
		.await?
	};
	if !exists {
		return Err(rivet_auth::errors::Auth::InsufficientPermissions.build());
	}
	Ok(())
}

pub fn no_store(mut response: axum::response::Response) -> axum::response::Response {
	response.headers_mut().insert(
		axum::http::header::CACHE_CONTROL,
		axum::http::HeaderValue::from_static("no-store"),
	);
	response
}

pub fn expiration(now: i64, seconds: u64, parent: Option<i64>) -> Result<i64> {
	let exp = now
		.checked_add(
			i64::try_from(seconds)
				.ok()
				.and_then(|s| s.checked_mul(1000))
				.ok_or_else(|| Signing::InvalidInput.build())?,
		)
		.ok_or_else(|| Signing::InvalidInput.build())?;
	let exp = parent.map_or(exp, |p| exp.min(p));
	if exp / 1000 <= now / 1000 {
		return Err(rivet_auth::errors::Auth::TokenExpired.build());
	}
	Ok(exp)
}

/// The authority owns a single consensus budget; a transport failure never falls back locally.
pub async fn charge(
	ctx: &ApiCtx,
	namespace_id: Id,
	charge: rivet_auth_jwt::ops::signing_quota::Charge,
) -> Result<()> {
	let input = rivet_auth_jwt::ops::signing_quota::Input {
		namespace_id,
		charge,
	};
	let result = if ctx.config().is_leader() {
		rivet_api_peer::auth::signing_quota::charge(ctx.clone().into(), (), (), input).await
	} else {
		rivet_api_util::request_remote_datacenter(
			ctx.config(),
			ctx.config().leader_dc()?.datacenter_label,
			"/auth/signing-quota",
			rivet_api_util::Method::POST,
			Option::<&()>::None,
			Some(&input),
		)
		.await
	};
	result.map_err(normalize_error)
}

pub fn error_response(error: anyhow::Error) -> axum::response::Response {
	use axum::response::IntoResponse;
	rivet_api_builder::ApiError::from(normalize_error(error)).into_response()
}

pub fn normalize_error(error: anyhow::Error) -> anyhow::Error {
	if error
		.chain()
		.find_map(|cause| cause.downcast_ref::<rivet_error::RivetError>())
		.is_some_and(|e| matches!(e.group(), "auth" | "signing"))
	{
		return error;
	}
	if let Some(raw) = error
		.chain()
		.find_map(|cause| cause.downcast_ref::<rivet_api_builder::RawErrorResponse>())
	{
		return match (raw.1.group.as_ref(), raw.1.code.as_ref()) {
			("signing", "invalid_input") => Signing::InvalidInput.build(),
			("signing", "payload_too_large") => Signing::PayloadTooLarge.build(),
			("signing", "throttled") => Signing::Throttled.build(),
			("signing", "conflict") => Signing::Conflict.build(),
			("signing", "not_found") => Signing::NotFound.build(),
			("auth", "invalid_token") => rivet_auth::errors::Auth::InvalidToken.build(),
			("auth", "token_expired") => rivet_auth::errors::Auth::TokenExpired.build(),
			("auth", "insufficient_permissions") => {
				rivet_auth::errors::Auth::InsufficientPermissions.build()
			}
			_ => Signing::Unavailable.build(),
		};
	}
	Signing::Unavailable.build()
}

pub async fn management_body<T: serde::de::DeserializeOwned>(
	request: axum::extract::Request,
) -> Result<T> {
	if request
		.headers()
		.get(axum::http::header::CONTENT_TYPE)
		.and_then(|h| h.to_str().ok())
		.and_then(|s| s.split(';').next())
		.map(str::trim)
		!= Some("application/json")
		|| request
			.headers()
			.contains_key(axum::http::header::CONTENT_ENCODING)
	{
		return Err(Signing::InvalidInput.build());
	}
	let bytes = tokio::time::timeout(
		std::time::Duration::from_secs(30),
		axum::body::to_bytes(request.into_body(), 4096),
	)
	.await
	.map_err(|_| Signing::Unavailable.build())?
	.map_err(|_| Signing::PayloadTooLarge.build())?;
	super::strict_json::parse(&bytes)
}
