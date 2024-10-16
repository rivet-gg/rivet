use api_helper::{anchor::WatchIndexQuery, ctx::Ctx};
use rivet_api::models;
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
		access: match server_config.rivet.access_kind {
			RivetAccessKind::Public => models::CloudBootstrapAccess::Public,
			RivetAccessKind::Private => models::CloudBootstrapAccess::Private,
			_ => bail!("invalid RIVET_ACCESS_KIND"),
		},
		domains: if let (Some(main), Some(cdn), Some(job)) = (
			config.server()?.rivet.domain.main,
			config.server()?.rivet.domain.cdn,
			config.server()?.rivet.domain.job,
		) {
			Some(Box::new(models::CloudBootstrapDomains {
				main: main.into(),
				cdn: cdn.into(),
				job: job.into(),
				opengb: None,
			}))
		} else {
			None
		},
		origins: Box::new(models::CloudBootstrapOrigins {
			hub: util::env::origin_hub().into(),
		}),
		captcha: Box::new(models::CloudBootstrapCaptcha {
			turnstile: server_config
				.turnstile
				.site_key_main
				.as_ref()
				.map(|site_key| Box::new(models::CloudBootstrapCaptchaTurnstile { site_key })),
		}),
		login_methods: Box::new(models::CloudBootstrapLoginMethods {
			access_token: server_config.rivet.acess_token_login,
			email: server_config.rivet.sendgrid.is_some(),
		}),
		deploy_hash: server_config.rivet.project_source_hash.clone(),
	})
}
