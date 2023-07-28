use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker(name = "db-schema-apply")]
async fn worker(ctx: OperationContext<db::msg::schema_apply::Message>) -> GlobalResult<()> {
	todo!();

	// TODO: Create TX
	// TODO: Read the db schema
	// TODO: Merge schema
	// TODO: Generate migration script
	// TODO: Run migration

	Ok(())
}
