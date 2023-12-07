use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use async_ssh2_tokio::client::{Client, AuthMethod, ServerCheckMethod};

#[worker(name = "cluster-server-install")]
async fn worker(ctx: &OperationContext<cluster::msg::server_install::Message>) -> GlobalResult<()> {
	let server_id = unwrap!(ctx.server_id).as_uuid();
	
	let (public_ip,) = sql_fetch_one!(
		[ctx, (String,)]
		"
		SELECT
			public_ip
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;

	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
    let auth_method = AuthMethod::with_key(&private_key_openssh, None);

	tracing::info!(?public_ip, "connecting ssh");

    let mut client = Client::connect(
        (public_ip.as_str(), 22),
        "root",
        auth_method,
		ServerCheckMethod::NoCheck,
        // ServerCheckMethod::DefaultKnownHostsFile,
    ).await?;

	tracing::info!("connected");

    let result = client.execute("echo Hello SSH").await?;
    if result.exit_status != 0 {
		tracing::info!(stdout=?result.stdout, stdout=?result.stderr, "failed to run script");
		bail!("failed to run script");
	}

	tracing::info!(stdout=?result.stdout, stdout=?result.stderr, "----------");

	Ok(())
}
