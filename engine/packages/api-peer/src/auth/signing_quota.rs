use anyhow::Result;
use rivet_api_builder::ApiCtx;
use rivet_auth_jwt::ops::signing_quota::Input;
#[tracing::instrument(skip_all)]
pub async fn charge(ctx: ApiCtx, _path: (), _query: (), input: Input) -> Result<()> {
	ctx.op(input).await.map_err(|error| {
		if error
			.chain()
			.find_map(|cause| cause.downcast_ref::<rivet_error::RivetError>())
			.is_some_and(|e| e.group() == "signing")
		{
			error
		} else {
			rivet_auth_policy::errors::Signing::Unavailable.build()
		}
	})
}
