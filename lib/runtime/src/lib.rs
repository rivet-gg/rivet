use std::{future::Future, sync::Once, time::Duration};

use thiserror::Error;

mod metrics;

static SETUP_TRACING: Once = Once::new();

#[derive(Error, Debug)]
pub enum Error {
	#[error("parse int: {0}")]
	ParseInt(#[from] std::num::ParseIntError),

	#[error("build tokio runtime: {0}")]
	BuildTokioRuntime(std::io::Error),
}

#[derive(Default)]
pub struct RunConfig {
	pub config: rivet_config::Config,
	pub customize_tokio_runtime: Option<Box<dyn FnOnce(&mut tokio::runtime::Builder) -> ()>>,
	pub pretty_logs: bool,
}

impl RunConfig {
	pub fn run<F: Future>(self, f: F) -> Result<F::Output, Error> {
		self.setup_tracing()?;

		// Build runtime
		let mut rt_builder = self.build_tokio_runtime_builder()?;
		if let Some(customize) = self.customize_tokio_runtime {
			customize(&mut rt_builder);
		}

		// Run future
		let rt = rt_builder.build().map_err(Error::BuildTokioRuntime)?;
		let output = rt.block_on(f);

		Ok(output)
	}

	fn setup_tracing(&self) ->Result<()>{
		SETUP_TRACING.call_once(|| {
			if self.pretty_logs {
				// Pretty print
				tracing_subscriber::fmt()
					.pretty()
					.with_max_level(tracing::Level::INFO)
					.init();
			} else {
				let fmt_filter = tracing_subscriber::filter::LevelFilter::INFO;

				if self.config.server()?.tokio.console_enable {
					// logfmt + tokio-console
					tracing_subscriber::registry()
						.with(
							console_subscriber::ConsoleLayer::builder()
								.retention(std::time::Duration::from_secs(60))
								.with_default_env()
								.spawn(),
						)
						.with(
							tracing_logfmt::builder()
								.layer()
								.with_filter(fmt_filter)
								.with_filter(fmt_filter),
						)
						.init();
				} else {
					// logfmt
					tracing_subscriber::registry()
						.with(tracing_logfmt::builder().layer().with_filter(fmt_filter))
						.init();
				}
			}
		})

		Ok(())
	}

	fn build_tokio_runtime_builder(&self) -> Result<tokio::runtime::Builder, Error> {
		let tokio_config = &self.config.server()?.tokio;

		let mut rt_builder = tokio::runtime::Builder::new_multi_thread();
		rt_builder.enable_all();

		rt_builder.on_thread_start(move || {
			metrics::TOKIO_THREAD_COUNT.inc();
		});
		rt_builder.on_thread_stop(move || {
			metrics::TOKIO_THREAD_COUNT.dec();
		});

		if let Ok(thread_stack_size) = tokio_config.thread_stack_size {
			rt_builder.thread_stack_size(thread_stack_size.parse()?);
		} else {
			// async-nats requires a fat stack
			rt_builder.thread_stack_size(8 * 1024 * 1024);
		}

		if let Ok(worker_threads) = tokio_config.worker_threads {
			rt_builder.worker_threads(worker_threads.parse()?);
		} else {
			// Default to 2 threads since this is likely running in a shared
			// context. If we constrain this task to use 100 MHz an 8 core system,
			// it will still spawn 8 threads needlessly.
			//
			// If a service is configured to use a dedicated core, Bolt will expose
			// the correct thread count.
			rt_builder.worker_threads(2);
		}

		if let Ok(max_blocking_threads) = tokio_config.max_blocking_threads {
			rt_builder.max_blocking_threads(max_blocking_threads.parse()?);
		}

		if let Ok(global_queue_interval) = tokio_config.global_queue_interval {
			rt_builder.global_queue_interval(global_queue_interval.parse()?);
		}

		if let Ok(event_interval) = tokio_config.event_interval {
			rt_builder.event_interval(event_interval.parse()?);
		}

		if let Ok(thread_keep_alive) = tokio_config.thread_keep_alive {
			rt_builder.thread_keep_alive(Duration::from_millis(thread_keep_alive.parse()?));
		}

		Ok(rt_builder)
	}
}

pub fn run<F: Future>(f: F) -> Result<F::Output, Error> {
	RunConfig::default().run(f)
}
