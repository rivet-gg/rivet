use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum EnvVarError {
	#[error("missing env var: {0}")]
	Missing(String),

	#[error("{0}")]
	Invalid(std::env::VarError),
}

lazy_static::lazy_static! {
	static ref HOST_API: Option<String> = std::env::var("RIVET_HOST_API").ok();
	static ref ORIGIN_API: Option<String> = std::env::var("RIVET_ORIGIN_API").ok();
	static ref ORIGIN_HUB: Option<String> = std::env::var("RIVET_ORIGIN_HUB").ok();
	static ref DNS_PROVIDER: Option<String> = std::env::var("RIVET_DNS_PROVIDER").ok();
	static ref CHIRP_SERVICE_NAME: Option<String> = std::env::var("CHIRP_SERVICE_NAME").ok();
	static ref BILLING: Option<RivetBilling> = std::env::var("RIVET_BILLING")
		.ok()
		.map(|x| serde_json::from_str(&x).expect("failed to parse billing"));
	static ref TEST_ID: Option<String> = std::env::var("RIVET_TEST_ID").ok();
}

pub fn test_id_param() -> Vec<types_proto::rivet::backend::job::Parameter> {
	TEST_ID
		.as_ref()
		.iter()
		.map(|x| types_proto::rivet::backend::job::Parameter {
			key: "rivet_test_id".to_string(),
			value: x.to_string(),
		})
		.collect()
}

/// The host for the API.
pub fn host_api() -> &'static str {
	match &*HOST_API {
		Some(x) => x.as_str(),
		None => panic!("{}", EnvVarError::Missing("RIVET_HOST_API".to_string())),
	}
}

/// The base domain for the API.
pub fn origin_api() -> &'static str {
	match &*ORIGIN_API {
		Some(x) => x.as_str(),
		None => panic!("{}", EnvVarError::Missing("RIVET_ORIGIN_API".to_string())),
	}
}

/// The base domain for the hub.
pub fn origin_hub() -> &'static str {
	match &*ORIGIN_HUB {
		Some(x) => x.as_str(),
		None => panic!("{}", EnvVarError::Missing("RIVET_ORIGIN_HUB".to_string())),
	}
}

pub fn dns_provider() -> Option<&'static str> {
	DNS_PROVIDER.as_ref().map(|x| x.as_str())
}

pub fn chirp_service_name() -> &'static str {
	match &*CHIRP_SERVICE_NAME {
		Some(x) => x.as_str(),
		None => panic!("{}", EnvVarError::Missing("CHIRP_SERVICE_NAME".to_string())),
	}
}

#[derive(Deserialize)]
pub struct RivetBilling {
	pub indie_price_id: String,
	pub studio_price_id: String,
}

pub fn billing() -> Option<&'static RivetBilling> {
	BILLING.as_ref()
}

/// The current stripe API token.
pub async fn stripe_secret_key() -> Result<String, EnvVarError> {
	read_secret(&["stripe", "secret_key"]).await
}

/// The current stripe webhook secret.
///
/// Secrets can be added at: https://dashboard.stripe.com/webhooks
///
/// Add the following events to the WebHook:
/// - invoice.payment_succeeded
/// - checkout.session.completed
/// - payment_intent.succeeded
pub async fn stripe_webhook_secret() -> Result<String, EnvVarError> {
	read_secret(&["stripe", "webhook_secret"]).await
}

pub mod cloudflare {
	use super::{var, EnvVarError};

	lazy_static::lazy_static! {
		static ref CLOUDFLARE_AUTH_TOKEN: Result<String, EnvVarError> = var("CLOUDFLARE_AUTH_TOKEN");
		static ref CLOUDFLARE_ACCOUNT_ID: Result<String, EnvVarError> = var("CLOUDFLARE_ACCOUNT_ID");
	}

	pub fn auth_token() -> &'static str {
		CLOUDFLARE_AUTH_TOKEN.as_ref().unwrap().as_str()
	}

	pub fn account_id() -> &'static str {
		CLOUDFLARE_ACCOUNT_ID.as_ref().unwrap().as_str()
	}

	pub mod zone {
		pub mod main {
			lazy_static::lazy_static! {
				static ref ID: Option<String> = std::env::var("CLOUDFLARE_ZONE_ID_MAIN").ok();
			}

			pub fn id() -> Option<&'static str> {
				ID.as_ref().map(|x| x.as_str())
			}
		}

		pub mod game {
			lazy_static::lazy_static! {
				static ref ID: Option<String> = std::env::var("CLOUDFLARE_ZONE_ID_GAME").ok();
			}

			pub fn id() -> Option<&'static str> {
				ID.as_ref().map(|x| x.as_str())
			}
		}

		pub mod job {
			lazy_static::lazy_static! {
				static ref ID: Option<String> = std::env::var("CLOUDFLARE_ZONE_ID_JOB").ok();
			}

			pub fn id() -> Option<&'static str> {
				ID.as_ref().map(|x| x.as_str())
			}
		}
	}
}

/// Reads a secret from the env.
///
/// This is marked as async so we have the flexibility to pull the secret from remote datasources.
pub async fn read_secret(key: &[impl AsRef<str>]) -> Result<String, EnvVarError> {
	var(secret_env_var_key(key))
}

pub async fn read_secret_opt(key: &[impl AsRef<str>]) -> Result<Option<String>, EnvVarError> {
	let env_var = read_secret(key).await;

	match env_var {
		Ok(v) => Ok(Some(v)),
		Err(var_error) => match var_error {
			EnvVarError::Missing(_) => Ok(None),
			EnvVarError::Invalid(_) => Err(var_error),
		},
	}
}

/// Name of env var holding a given secret.
pub fn secret_env_var_key(key: &[impl AsRef<str>]) -> String {
	key.iter()
		.map(|x| x.as_ref().to_uppercase())
		.collect::<Vec<_>>()
		.join("_")
}

pub fn var(name: impl AsRef<str>) -> Result<String, EnvVarError> {
	let env_var = std::env::var(name.as_ref());
	match env_var {
		Ok(v) => Ok(v),
		Err(var_error) => match var_error {
			std::env::VarError::NotPresent => Err(EnvVarError::Missing(name.as_ref().to_string())),
			_ => Err(EnvVarError::Invalid(var_error)),
		},
	}
}
