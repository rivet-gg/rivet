use rivet_metrics::{prometheus::*, REGISTRY};

lazy_static::lazy_static! {
	// MARK: CRDB
	pub static ref CRDB_POOL_SIZE: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"crdb_pool_conn_size",
		"Number of SQL connections in the pool.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CRDB_POOL_NUM_IDLE: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"crdb_pool_num_idle",
		"Number of idle SQL connections in the pool.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();

	// MARK: Postgres
	pub static ref POSTGRES_POOL_SIZE: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"postgres_pool_conn_size",
		"Number of SQL connections in the pool.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref POSTGRES_POOL_NUM_IDLE: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"postgres_pool_num_idle",
		"Number of idle SQL connections in the pool.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();

	// MARK: Cassandra
	pub static ref CASSANDRA_LATENCY_AVERAGE: GaugeVec = register_gauge_vec_with_registry!(
		"cassandra_latency_average",
		"Average latency of queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_LATENCY_P95: GaugeVec = register_gauge_vec_with_registry!(
		"cassandra_latency_p95",
		"P95 latency of queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_LATENCY_P99: GaugeVec = register_gauge_vec_with_registry!(
		"cassandra_latency_p99",
		"P99 latency of queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_ERRORS_NUM: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"cassandra_errors_num",
		"Counter for errors occurred in nonpaged queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_QUERIES_NUM: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"cassandra_queries_num",
		"Counter for nonpaged queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_ERRORS_ITER_NUM: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"cassandra_errors_iter_num",
		"Counter for errors occurred in paged queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_QUERIES_ITER_NUM: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"cassandra_queries_iter_num",
		"Counter for pages requested in paged queries.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
	pub static ref CASSANDRA_RETRIES_NUM: IntGaugeVec = register_int_gauge_vec_with_registry!(
		"cassandra_retries_num",
		"Counter measuring how many times a retry policy has decided to retry a query.",
		&["db_name"],
		*REGISTRY,
	)
	.unwrap();
}
