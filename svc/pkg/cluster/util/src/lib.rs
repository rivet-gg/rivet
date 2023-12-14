use types::rivet::backend;
use uuid::Uuid;

pub fn server_name(provider_datacenter_id: &str, pool_type: backend::cluster::PoolType) -> String {
	let ns = rivet_util::env::namespace();
	let pool_type_str = match pool_type {
		backend::cluster::PoolType::Job => "job",
		backend::cluster::PoolType::Gg => "gg",
		backend::cluster::PoolType::Ats => "ats",
	};

	format!("{ns}-{provider_datacenter_id}-{pool_type_str}")
}

pub fn full_server_name(
	provider_datacenter_id: &str,
	pool_type: backend::cluster::PoolType,
	server_id: Uuid,
) -> String {
	format!(
		"{}-{server_id}",
		server_name(provider_datacenter_id, pool_type),
	)
}
