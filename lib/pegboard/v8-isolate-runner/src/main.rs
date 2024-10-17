use std::{fs, path::Path};

use anyhow::*;
use deno_runtime::deno_core::JsRuntime;
use notify::{
	event::{AccessKind, AccessMode},
	EventKind, RecursiveMode, Watcher,
};
use utils::{var, watcher};
use uuid::Uuid;

mod config;
mod isolate;
mod log_shipper;
mod throttle;
mod utils;

fn main() -> Result<()> {
	let working_path = std::env::args()
		.skip(1)
		.next()
		.context("`working_path` arg required")?;
	let working_path = Path::new(&working_path);

	// Write PID to file
	fs::write(
		working_path.join("pid"),
		std::process::id().to_string().as_bytes(),
	)?;

	let (mut watcher, rx) = watcher()?;

	// Watch actors
	let actors_path = var("ACTORS_PATH")?;
	let actors_path = Path::new(&actors_path);
	watcher.watch(actors_path, RecursiveMode::Recursive)?;

	// Explicitly start runtime on current thread
	JsRuntime::init_platform(None, false);

	println!("Watching for actor creation at {}", actors_path.display());

	loop {
		let res = rx.recv()??;

		// Wait for creation of config
		if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = res.kind {
			if let Some(config_path) = res.paths.iter().find(|p| p.ends_with("config.json")) {
				let actor_path = config_path
					.parent()
					.context("empty `config_path`")?
					.to_path_buf();

				// Extract actor id from path
				let actor_id = Uuid::parse_str(
					&actor_path
						.iter()
						.last()
						.context("empty `actor_path`")?
						.to_string_lossy()
						.to_string(),
				)
				.context("invalid actor id")?;

				std::thread::Builder::new()
					.name(actor_id.to_string())
					.spawn(move || {
						if let Err(err) = isolate::run(actor_id, actor_path) {
							eprintln!("Isolate thread failed ({actor_id}):\n{err:?}");
						}
					})?;
			}
		}
	}
}
