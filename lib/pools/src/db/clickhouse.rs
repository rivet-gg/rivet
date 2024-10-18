use rivet_config::Config;
use std::time::Duration;

use crate::Error;

pub type ClickHousePool = clickhouse::Client;

#[tracing::instrument(skip(config))]
pub fn setup(config: Config) -> Result<Option<ClickHousePool>, Error> {
	if let Some(clickhouse) = &config.server().map_err(Error::Global)?.clickhouse {
		tracing::info!("clickhouse connecting");

		// Build HTTP client
		let mut http_connector = hyper::client::connect::HttpConnector::new();
		http_connector.enforce_http(false);
		http_connector.set_keepalive(Some(Duration::from_secs(15)));
		let https_connector = hyper_tls::HttpsConnector::new_with_connector(http_connector);
		let http_client = hyper::Client::builder()
			.pool_idle_timeout(Duration::from_secs(2))
			.build(https_connector);

		// Build ClickHouse client
		let parsed_url =
			url::Url::parse(&clickhouse.url.read()).map_err(Error::BuildClickHouseUrl)?;
		let mut client = clickhouse::Client::with_http_client(http_client)
			.with_url(&*clickhouse.url.read())
			.with_user(parsed_url.username());
		if let Some(password) = parsed_url.password() {
			client = client.with_password(password);
		}

		Ok(Some(client))
	} else {
		Ok(None)
	}
}
