use std::time::Duration;

use anyhow::*;
use notify::{Config, Event, RecommendedWatcher, Watcher};

pub fn var(name: &str) -> Result<String> {
	std::env::var(name).context(name.to_string())
}

/// Creates an async file watcher.
pub fn watcher() -> Result<(
	RecommendedWatcher,
	std::sync::mpsc::Receiver<notify::Result<Event>>,
)> {
	let (tx, rx) = std::sync::mpsc::sync_channel(1);

	// Automatically select the best implementation for your platform.
	let watcher = RecommendedWatcher::new(
		move |res| {
			let tx = tx.clone();

			let _ = tx.send(res);
		},
		Config::default().with_poll_interval(Duration::from_secs(2)),
	)?;

	Ok((watcher, rx))
}
