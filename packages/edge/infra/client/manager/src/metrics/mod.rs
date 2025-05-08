use prometheus::*;

mod buckets;
mod registry;
mod server;

pub use buckets::{BUCKETS, MICRO_BUCKETS};
pub use registry::REGISTRY;
pub use server::run_standalone;

lazy_static::lazy_static! {
	pub static ref PACKET_RECV_TOTAL: IntCounterVec = register_int_counter_vec_with_registry!(
		"packet_recv_total",
		"Total number of packets received.",
		&[],
		*REGISTRY,
	).unwrap();

	pub static ref PACKET_SEND_TOTAL: IntCounterVec = register_int_counter_vec_with_registry!(
		"packet_send_total",
		"Total number of packets sent.",
		&[],
		*REGISTRY,
	).unwrap();

	pub static ref SQL_ERROR: IntCounterVec = register_int_counter_vec_with_registry!(
		"sql_error",
		"An SQL error occurred.",
		&["error"],
		*REGISTRY,
	).unwrap();

	pub static ref DOWNLOAD_IMAGE_RATE: GaugeVec = register_gauge_vec_with_registry!(
		"download_image_rate",
		"Rate of image download in bytes/sec",
		&["bucket"],
		*REGISTRY,
	).unwrap();

	// MARK: Actor setup step duration metrics
	pub static ref SETUP_TOTAL_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_total_duration",
		"Duration of the complete actor setup process",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_MAKE_FS_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_make_fs_duration",
		"Duration of fs creation step",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_DOWNLOAD_IMAGE_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_download_image_duration",
		"Duration of image download step",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_BIND_PORTS_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_bind_ports_duration",
		"Duration of port binding step",
		MICRO_BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_CNI_NETWORK_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_cni_network_duration",
		"Duration of CNI network setup step",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_OCI_BUNDLE_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_oci_bundle_duration",
		"Duration of OCI bundle setup step",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref SETUP_PARALLEL_TASKS_DURATION: Histogram = register_histogram_with_registry!(
		"actor_setup_parallel_tasks_duration",
		"Duration of parallel setup tasks (image download/fs + ports/network)",
		BUCKETS.to_vec(),
		*REGISTRY,
	).unwrap();

	pub static ref IMAGE_DOWNLOAD_REQUEST_TOTAL: IntCounter = register_int_counter_with_registry!(
		"image_download_request_total",
		"Total number of download requests.",
		*REGISTRY,
	).unwrap();

	pub static ref IMAGE_DOWNLOAD_CACHE_MISS_TOTAL: IntCounter = register_int_counter_with_registry!(
		"image_download_cache_miss_total",
		"Total number of download requests that missed cache.",
		*REGISTRY,
	).unwrap();

	pub static ref IMAGE_CACHE_COUNT: IntGauge = register_int_gauge_with_registry!(
		"image_cache_count",
		"Total number of images currently in cache.",
		*REGISTRY,
	).unwrap();

	pub static ref IMAGE_CACHE_SIZE: IntGauge = register_int_gauge_with_registry!(
		"image_cache_size",
		"Total byte size of cache images folder.",
		*REGISTRY,
	).unwrap();
}
