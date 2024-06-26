use futures_util::FutureExt;
use proto::backend::pkg::*;
use rivet_operation::prelude::*;

#[operation(name = "ds-server-delete")]
pub async fn handle(
	ctx: OperationContext<dynamic_servers::server_delete::Request>,
) -> GlobalResult<dynamic_servers::server_delete::Response> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	rivet_pools::utils::crdb::tx(&ctx.crdb().await?, |tx| {
		let ctx = ctx.clone();

		async move {
			sql_execute!(
				[ctx, @tx tx]
				"
				UPDATE db_dynamic_servers.servers
				SET delete_ts = $2
				WHERE
					server_id = $1
					AND delete_ts IS NULL
				",
				server_id,
				ctx.ts(),
			)
			.await?;

			Ok(())
		}
		.boxed()
	})
	.await?;

	Ok(dynamic_servers::server_delete::Response {
		server_id: Some(server_id.into()),
	})
}
