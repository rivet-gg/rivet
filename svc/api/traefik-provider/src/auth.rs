use api_helper::auth::{ApiAuth, AuthRateLimitCtx};
use proto::claims::Claims;
use rivet_operation::prelude::*;

/// Information derived from the authentication middleware.
pub struct Auth {
	config: rivet_config::Config,
	_claims: Option<Claims>,
}

#[async_trait]
impl ApiAuth for Auth {
	async fn new(
		config: rivet_config::Config,
		_api_token: Option<String>,
		rate_limit_ctx: AuthRateLimitCtx<'_>,
	) -> GlobalResult<Auth> {
		Self::rate_limit(&config, rate_limit_ctx).await?;

		Ok(Auth {
			config,
			_claims: None,
		})
	}

	async fn rate_limit(
		_config: &rivet_config::Config,
		_rate_limit_ctx: AuthRateLimitCtx<'_>,
	) -> GlobalResult<()> {
		Ok(())
	}
}

impl Auth {
	pub async fn token(&self, token: &str) -> GlobalResult<()> {
		ensure_eq_with!(
			token,
			self.config
				.server()?
				.rivet
				.tokens
				.api_traefik_provider
				.read()
				.as_str(),
			API_FORBIDDEN,
			reason = "Invalid token",
		);

		Ok(())
	}
}
