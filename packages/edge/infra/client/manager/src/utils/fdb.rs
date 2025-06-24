use std::{ops::Deref, path::Path, result::Result::Ok, sync::Arc};

use anyhow::*;
use foundationdb as fdb;
use pegboard_config::Config;
use service_discovery::ServiceDiscovery;
use tokio::fs;

// TODO: Copied from rivet_pools
#[derive(Clone)]
pub struct FdbPool {
	db: Arc<fdb::Database>,
	_sd: Option<Arc<ServiceDiscovery>>,
	// Prevent dropping temp file
	_connection_file: Arc<tempfile::NamedTempFile>,
}

impl Deref for FdbPool {
	type Target = Arc<fdb::Database>;

	fn deref(&self) -> &Self::Target {
		&self.db
	}
}

impl FdbPool {
	#[tracing::instrument(skip(config))]
	pub async fn new(config: &Config) -> Result<FdbPool> {
		let temp_file = tempfile::NamedTempFile::new()?;
		let temp_path = temp_file.path().to_path_buf();

		let fdb_config = &config.client.foundationdb;

		let sd = match &fdb_config.addresses {
			pegboard_config::Addresses::Dynamic { fetch_endpoint } => {
				let sd = ServiceDiscovery::new(fetch_endpoint.clone());

				// Initial fetch
				let servers = sd.fetch().await.context("failed to fetch services")?;
				let joined = servers
					.into_iter()
					.filter_map(|server| server.lan_ip)
					.map(|lan_ip| format!("{lan_ip}:4500"))
					.collect::<Vec<_>>()
					.join(",");
				write_connection_file(&fdb_config, &temp_path, &joined).await?;

				let fdb_config = config.client.foundationdb.clone();
				sd.start(move |servers| {
					let temp_path = temp_path.clone();
					let fdb_config = fdb_config.clone();
					async move {
						let joined = servers
							.into_iter()
							.filter_map(|server| server.lan_ip)
							.map(|lan_ip| format!("{lan_ip}:4500"))
							.collect::<Vec<_>>()
							.join(",");

						write_connection_file(&fdb_config, &temp_path, &joined).await?;

						anyhow::Ok(())
					}
				});

				Some(sd)
			}
			pegboard_config::Addresses::Static(addresses) => {
				let joined = addresses.join(",");
				write_connection_file(&fdb_config, &temp_path, &joined).await?;

				None
			}
		};

		// Start network
		fdb_util::init(temp_file.path());

		let fdb_handle = fdb_util::handle(&temp_file.path())?;

		tracing::debug!(config_file_path=%temp_file.path().display(), "fdb started");

		Ok(FdbPool {
			db: Arc::new(fdb_handle),
			_sd: sd,
			_connection_file: Arc::new(temp_file),
		})
	}
}

async fn write_connection_file(
	fdb_config: &pegboard_config::FoundationDb,
	temp_path: &Path,
	joined: &str,
) -> Result<(), std::io::Error> {
	let connection = format!(
		"{cluster_description}:{cluster_id}@{joined}",
		cluster_description = fdb_config.cluster_description,
		cluster_id = fdb_config.cluster_id,
	);

	fs::write(temp_path, connection.as_bytes()).await?;

	Ok(())
}
