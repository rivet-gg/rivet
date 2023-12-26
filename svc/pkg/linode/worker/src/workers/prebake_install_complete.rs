use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use util_linode::api;

#[derive(sqlx::FromRow)]
struct PrebakeServer {
	variant: String,
	linode_id: i64,
	disk_id: i64,
}

#[worker(name = "linode-prebake-install-complete")]
async fn worker(
	ctx: &OperationContext<linode::msg::prebake_install_complete::Message>,
) -> GlobalResult<()> {
	let crdb = ctx.crdb().await?;
	let prebake_server = sql_fetch_one!(
		[ctx, PrebakeServer, &crdb]
		"
		SELECT variant, linode_id, disk_id
		FROM db_cluster.server_images_linode_misc
		WHERE public_ip = $1
		",
		&ctx.ip,
	)
	.await?;

	// Build HTTP client
	let client = util_linode::Client::new().await?;

	// Shut down server before creating custom image
	api::shut_down(&client, prebake_server.linode_id).await?;

	let create_image_res =
		api::create_custom_image(&client, &prebake_server.variant, prebake_server.disk_id).await?;

	// Write image id
	sql_execute!(
		[ctx, &crdb]
		"
		UPDATE db_cluster.server_images_linode_misc
		SET image_id = $2
		WHERE variant = $1
		",
		&prebake_server.variant,
		create_image_res.id,
	)
	.await?;

	Ok(())
}
