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

// Use the hash of the server install script in the image variant so that if the install scripts are updated
// we wont be using the old image anymore
const CLUSTER_SERVER_INSTALL_HASH: &str = include_str!("../gen/hash.txt");

pub fn image_variant(
	provider: backend::cluster::Provider,
	provider_datacenter_id: &str,
	pool_type: backend::cluster::PoolType,
) -> String {
	let ns = rivet_util::env::namespace();
	let provider_str = match provider {
		backend::cluster::Provider::Linode => "linode",
	};
	let pool_type_str = match pool_type {
		backend::cluster::PoolType::Job => "job",
		backend::cluster::PoolType::Gg => "gg",
		backend::cluster::PoolType::Ats => "ats",
	};

	format!("{ns}-{CLUSTER_SERVER_INSTALL_HASH}-{provider_str}-{provider_datacenter_id}-{pool_type_str}")
}
