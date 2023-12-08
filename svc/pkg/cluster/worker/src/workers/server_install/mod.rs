use std::{io::Read, net::TcpStream};

use chirp_worker::prelude::*;
use proto::backend::pkg::*;
use ssh2::Session;

mod install_scripts;
use install_scripts::ServerCtx;

#[worker(name = "cluster-server-install")]
async fn worker(ctx: &OperationContext<cluster::msg::server_install::Message>) -> GlobalResult<()> {
	let server_id = unwrap!(ctx.server_id).as_uuid();
	
	let row = sql_fetch_optional!(
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

	// Fail gracefully
	let Some((public_ip,)) = row else {
		tracing::error!(?server_id, "attempting to install scripts on a server that doesn't exist");
		return Ok(());
	};
	
	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let install_script = install_scripts::gen(todo!()).await?;

	// Spawn blocking thread for ssh (no async support)
	tokio::task::spawn_blocking(move || {
		tracing::info!(?public_ip, "connecting ssh");
		let tcp = TcpStream::connect((public_ip.as_str(), 22))?;
		let mut sess = Session::new()?;
		sess.set_tcp_stream(tcp);
		sess.handshake()?;

		tracing::info!("connected");

		sess.userauth_pubkey_memory("root", None, &private_key_openssh, None);
		
		tracing::info!("authenticated");

		let mut channel = sess.channel_session()?;
		channel.exec(&install_script)?;

		if channel.exit_status()? != 0 {
			let mut stdout = String::new();
			channel.read_to_string(&mut stdout)?;
			let mut stderr = String::new();
			channel.stderr().read_to_string(&mut stderr)?;
			channel.wait_close()?;

			tracing::info!(%stdout, %stderr, "failed to run script");
			bail!("failed to run script");
		}

		tracing::info!("script successful");

		channel.wait_close()?;

		GlobalResult::Ok(())
	}).await?;

	Ok(())
}
