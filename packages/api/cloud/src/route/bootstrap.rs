use api_helper::{anchor::WatchIndexQuery, ctx::Ctx};
use rivet_api::models;
use rivet_config::config::rivet::RivetAccessKind;
use rivet_operation::prelude::*;

use crate::auth::Auth;

// MARK: GET /bootstrap
pub async fn get(
	ctx: Ctx<Auth>,
	_watch_index_query: WatchIndexQuery,
) -> GlobalResult<models::CloudBootstrapResponse> {
	build_bootstrap_data(ctx.config()).await
}

pub async fn build_bootstrap_data(
	config: &rivet_config::Config,
) -> GlobalResult<models::CloudBootstrapResponse> {
	let server_config = config.server()?;

	Ok(models::CloudBootstrapResponse {
		cluster: models::CloudBootstrapCluster::Oss,
		access: match server_config.rivet.auth.access_kind {
			RivetAccessKind::Public => models::CloudBootstrapAccess::Public,
			RivetAccessKind::Private => models::CloudBootstrapAccess::Private,
		},
		domains: if let (Some(cdn), Some(job)) = (
			config.server()?.rivet.dns()?.domain_cdn.clone(),
			config.server()?.rivet.dns()?.domain_job.clone(),
		) {
			Some(Box::new(models::CloudBootstrapDomains {
				cdn: cdn.into(),
				job: job.into(),
				opengb: None,
			}))
		} else {
			None
		},
		origins: Box::new(models::CloudBootstrapOrigins {
			hub: config.server()?.rivet.hub.public_origin.to_string(),
		}),
		captcha: Box::new(models::CloudBootstrapCaptcha {
			turnstile: server_config
				.turnstile
				.as_ref()
				.and_then(|x| x.main_site_key.clone())
				.map(|site_key| Box::new(models::CloudBootstrapCaptchaTurnstile { site_key })),
		}),
		login_methods: Box::new(models::CloudBootstrapLoginMethods {
			access_token: server_config.rivet.auth.access_token_login,
			email: server_config.sendgrid.is_some(),
		}),
		deploy_hash: rivet_env::source_hash().to_string(),
	})
}
