use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

#[operation(name = "db-query-run")]
pub async fn handle(
	ctx: OperationContext<db::query_run::Request>,
) -> GlobalResult<db::query_run::Response> {
	let vt = ctx.postgres("db-db-data").await?;

	sqlx::query_as::<_, (i64,)>("SELECT 1::INT8")
		.fetch_one(&vt)
		.await?;

	Ok(db::query_run::Response {})
}
