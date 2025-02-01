use foundationdb as fdb;

pub mod env;
pub mod server;

pub fn subspace() -> fdb::tuple::Subspace {
	fdb::tuple::Subspace::all().subspace(&("ds"))
}
