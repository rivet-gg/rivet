use std::{path::Path, io::{Write, Read}, net::TcpStream};

use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};
use ssh2::Session;

mod install_scripts;
use install_scripts::ServerCtx;

#[derive(sqlx::FromRow)]
struct Server {
	datacenter_id: Uuid,
	cluster_id: Uuid,
	pool_type: i64,
	public_ip: String,
	vlan_ip: String,
	cloud_destroy_ts: Option<i64>,
}

#[worker(name = "cluster-server-install", timeout = 200)]
async fn worker(ctx: &OperationContext<cluster::msg::server_install::Message>) -> GlobalResult<()> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();

	// Check for stale message
	if ctx.req_dt() > util::duration::hours(1) {
		tracing::warn!("discarding stale message");

		return Ok(());
	}
	
	let server = sql_fetch_one!(
		[ctx, Server]
		"
		SELECT
			datacenter_id, cluster_id, pool_type, public_ip, vlan_ip, cloud_destroy_ts
		FROM db_cluster.servers
		WHERE server_id = $1
		",
		server_id,
	)
	.await?;

	if server.cloud_destroy_ts.is_some() {
		tracing::info!("server marked for deletion, not installing");
		return Ok(());
	}
	
	let pool_type = unwrap!(backend::cluster::PoolType::from_i32(server.pool_type as i32));

	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![server.datacenter_id.into()],
	}).await?;
	let datacenter = unwrap!(datacenter_res.datacenters.first());

	let server_ctx = ServerCtx {
		server_id,
		datacenter_id: server.datacenter_id,
		cluster_id: server.cluster_id,
		provider_datacenter_id: datacenter.provider_datacenter_id.clone(),
		name: util_cluster::server_name(&datacenter.provider_datacenter_id, pool_type),
		pool_type,
		vlan_ip: server.vlan_ip,
	};
	
	let public_ip = server.public_ip;
	let private_key_openssh =
		util::env::read_secret(&["ssh", "server", "private_key_openssh"]).await?;
	let install_script = install_scripts::gen(&server_ctx).await?;

	// Spawn blocking thread for ssh (no async support)
	tokio::task::spawn_blocking(move || {
		tracing::info!(?server_id, %public_ip, "connecting over ssh");
		let tcp = TcpStream::connect((public_ip.as_str(), 22))?;
		let mut sess = Session::new()?;
		sess.set_tcp_stream(tcp);
		
		if let Err(err) = sess.handshake() {
			tracing::error!(?server_id, ?err, "failed to connect over ssh");
			retry_bail!("failed to connect over ssh");
		}

		tracing::info!(?server_id, "connected");

		sess.userauth_pubkey_memory("root", None, &private_key_openssh, None)?;
		
		tracing::info!("authenticated");

		tracing::info!("writing script");

		let install_script = install_script.as_bytes();
		let mut script_file = sess.scp_send(
			Path::new("/tmp/install.sh"),
			0o644,
			install_script.len() as u64,
			None
		)?;
		
		// Write script in chunks
		let mut idx = 0;
		loop {
			let start = idx;
			let end = (idx + 1024).min(install_script.len());

			script_file.write_all(&install_script[start..end])?;
			
			idx = end;
			if idx >= install_script.len() {
				break;
			}
		}
		// Close the channel and wait for the whole content to be transferred
		script_file.send_eof()?;
		script_file.wait_eof()?;
		script_file.close()?;
		script_file.wait_close()?;

		tracing::info!("executing script");

		let mut channel = sess.channel_session()?;
		channel.exec("chmod +x /tmp/install.sh && /tmp/install.sh || rm /tmp/install.sh")?;

		let mut stdout = String::new();
		channel.read_to_string(&mut stdout)?;
		let mut stderr = String::new();
		channel.stderr().read_to_string(&mut stderr)?;

		channel.wait_close()?;

		if channel.exit_status()? != 0 {
			tracing::info!(%stdout, %stderr, "failed to run script");
			bail!("failed to run script");
		}

		tracing::info!("script successful");

		GlobalResult::Ok(())
	}).await??;

	msg!([ctx] cluster::msg::server_install_complete(server_id) {
		server_id: ctx.server_id,
	}).await?;

	Ok(())
}
