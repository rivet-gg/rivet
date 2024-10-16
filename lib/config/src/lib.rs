use ::config as config_loader;
use global_error::prelude::*;
use std::{ops::Deref, path::Path, sync::Arc};

pub mod config;
pub mod paths;
pub mod secret;

struct ConfigData {
	config: config::Root,
}

#[derive(Clone)]
pub struct Config(Arc<ConfigData>);

impl Config {
	pub async fn load<P: AsRef<Path>>(paths: &[P]) -> GlobalResult<Self> {
		let mut settings = config_loader::Config::builder().add_source(
			config_loader::Environment::with_prefix("RIVET")
				.convert_case(config_loader::Case::Snake),
		);

		if paths.is_empty() {
			// Add default config directory
			settings = add_directory_source(settings, paths::system_config_dir())?;
		} else {
			for path in paths {
				let path = path.as_ref();
				if path.is_file() {
					settings = settings.add_source(config_loader::File::from(path));
				} else if path.is_dir() {
					settings = add_directory_source(settings, path)?;
				} else {
					bail!("Invalid path: {}", path.display());
				}
			}
		}

		// Read config
		let config = unwrap!(
			unwrap!(settings.build(), "failed to build config").try_deserialize::<config::Root>(),
			"failed to deserialize config"
		);

		Ok(Self(Arc::new(ConfigData { config })))
	}
}

impl Deref for Config {
	type Target = config::Root;

	fn deref(&self) -> &Self::Target {
		&self.0.config
	}
}

/// Loads all config files from a directory.
fn add_directory_source<P: AsRef<Path>>(
	mut settings: config_loader::ConfigBuilder<config_loader::builder::DefaultState>,
	dir: P,
) -> GlobalResult<config_loader::ConfigBuilder<config_loader::builder::DefaultState>> {
	let dir = dir.as_ref();
	let glob_pattern = dir.join("**/*.{json,yaml,yml}");
	let glob_pattern = unwrap!(glob_pattern.to_str());
	let entries = unwrap!(glob::glob(glob_pattern), "failed to glob directory");
	for entry in entries.flatten() {
		settings = settings.add_source(config_loader::File::from(entry));
	}

	Ok(settings)
}
