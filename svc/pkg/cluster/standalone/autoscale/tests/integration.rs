use chirp_worker::prelude::*;

use ::cluster_autoscale::run_from_env;

#[tokio::test(flavor = "multi_thread")]
async fn basic() {
	tracing_subscriber::fmt()
		.json()
		.with_max_level(tracing::Level::INFO)
		.with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
		.init();

	let pools = rivet_pools::from_env("cluster-autoscale-test")
		.await
		.unwrap();

	run_from_env(pools).await.unwrap();
}
