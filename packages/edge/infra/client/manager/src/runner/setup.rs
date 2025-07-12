use std::{
	collections::HashMap,
	os::unix::{fs::PermissionsExt, process::CommandExt},
	path::{Path, PathBuf},
	process::Stdio,
	result::Result::{Err, Ok},
	sync::Arc,
	time::Instant,
};

use anyhow::*;
use indoc::indoc;
use nix::{
	sys::wait::{waitpid, WaitStatus},
	unistd::{fork, pipe, read, setsid, write, ForkResult, Pid},
};
use pegboard::protocol;
use rand::Rng;
use serde_json::json;
use sqlx::Acquire;
use strum::FromRepr;
use tokio::{
	fs::{self, File},
	net::UnixListener,
	process::Command,
};
use uuid::Uuid;

use super::{oci_config, Runner};
use crate::{ctx::Ctx, utils};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum Comms {
	Basic = 0,
	Socket = 1,
}

impl Runner {
	pub async fn setup(
		self: &Arc<Self>,
		ctx: &Arc<Ctx>,
	) -> Result<protocol::HashableMap<String, protocol::ProxiedPort>> {
		let setup_start_instant = std::time::Instant::now();
		tracing::info!(runner_id=?self.runner_id, "setting up runner");

		tracing::info!(runner_id=?self.runner_id, "creating runner working directory");

		let runner_path = ctx.runner_path(self.runner_id);
		fs::create_dir(&runner_path)
			.await
			.context("failed to create runner dir")?;

		tracing::info!(runner_id=?self.runner_id, "starting setup tasks");
		let tasks_start_instant = Instant::now();

		let (_, ports) = tokio::try_join!(
			async {
				self.download_image(&ctx).await?;
				self.make_fs(&ctx).await?;

				Result::<(), anyhow::Error>::Ok(())
			},
			async {
				let ports = self.bind_ports(ctx).await?;

				if let protocol::NetworkMode::Bridge = self.config.network_mode {
					self.setup_cni_network(&ctx, &ports).await?;
				}

				Ok(ports)
			},
		)?;

		crate::metrics::SETUP_PARALLEL_TASKS_DURATION
			.observe(tasks_start_instant.elapsed().as_secs_f64());
		tracing::info!(
			runner_id=?self.runner_id,
			"setup tasks completed"
		);

		self.start_socket(ctx).await?;

		tracing::info!(runner_id=?self.runner_id, "setting up runtime environment");
		self.setup_oci_bundle(&ctx, &ports).await?;

		crate::metrics::SETUP_TOTAL_DURATION.observe(setup_start_instant.elapsed().as_secs_f64());
		tracing::info!(runner_id=?self.runner_id, "runner setup completed");

		Ok(ports)
	}

	pub async fn make_fs(&self, ctx: &Ctx) -> Result<()> {
		let timer = Instant::now();
		let runner_path = ctx.runner_path(self.runner_id);

		let fs_img_path = runner_path.join("fs.img");
		let fs_path = runner_path.join("fs");
		let fs_upper_path = fs_path.join("upper");
		let fs_work_path = fs_path.join("work");
		let image_path = ctx.image_path(self.config.image.id);

		tracing::info!(runner_id=?self.runner_id, "creating fs");

		fs::create_dir(&fs_path)
			.await
			.context("failed to create runner fs dir")?;

		if ctx.config().runner.use_mounts() {
			tracing::info!(runner_id=?self.runner_id, "creating disk image");
			// Create a zero-filled file
			let fs_img = File::create(&fs_img_path)
				.await
				.context("failed to create disk image")?;
			fs_img
				.set_len(self.config.resources.disk as u64 * 1024 * 1024)
				.await
				.context("failed to set disk image length")?;

			tracing::info!(runner_id=?self.runner_id, "formatting disk image");
			// Format file as ext4
			let cmd_out = Command::new("mkfs.ext4")
				.arg(&fs_img_path)
				.output()
				.await
				.context("failed to run `mkfs.ext4`")?;

			ensure!(
				cmd_out.status.success(),
				"failed `mkfs.ext4` command\n{}",
				std::str::from_utf8(&cmd_out.stderr)?
			);

			tracing::info!(runner_id=?self.runner_id, "mounting disk image");

			// Mount fs img as loop mount
			let cmd_out = Command::new("mount")
				.arg("-o")
				.arg("loop")
				.arg(&fs_img_path)
				.arg(&fs_path)
				.output()
				.await
				.context("failed to run `mount`")?;

			ensure!(
				cmd_out.status.success(),
				"failed `mount` command\n{}",
				std::str::from_utf8(&cmd_out.stderr)?
			);

			// Create folders on disk
			fs::create_dir(&fs_upper_path)
				.await
				.context("failed to create actor fs upper dir")?;
			fs::create_dir(&fs_work_path)
				.await
				.context("failed to create actor fs work dir")?;

			tracing::info!(runner_id=?self.runner_id, "mounting overlay");

			ensure!(
				fs::metadata(&image_path).await.is_ok(),
				"image dir does not exist"
			);

			// Overlay mount setup:
			// lowerdir = extracted build in manager's builds cache folder
			// upperdir = {actor dir}/fs/upper folder
			// workdir = {actor dir}/fs/work folder
			// merged dir = also fs/upper folder, it mounts to the upperdir
			let cmd_out = Command::new("mount")
				.arg("-t")
				.arg("overlay")
				// Arbitrary device name
				.arg(self.runner_id.to_string())
				.arg("-o")
				.arg(format!(
					"lowerdir={},upperdir={},workdir={}",
					image_path.display(),
					fs_upper_path.display(),
					fs_work_path.display()
				))
				.arg(&fs_upper_path)
				.output()
				.await
				.context("failed to run overlay `mount`")?;

			ensure!(
				cmd_out.status.success(),
				"failed overlay `mount` command\n{}",
				std::str::from_utf8(&cmd_out.stderr)?
			);
		} else {
			// Create folder on host
			fs::create_dir(&fs_upper_path)
				.await
				.context("failed to create actor fs upper dir")?;

			tracing::info!(runner_id=?self.runner_id, "copying image contents to fs");

			// Copy everything from the image (lowerdir) to the upperdir
			utils::copy_dir_all(image_path, &fs_upper_path)
				.await
				.context("failed to copy image contents to fs upper dir")?;
		}

		let duration = timer.elapsed().as_secs_f64();
		crate::metrics::SETUP_MAKE_FS_DURATION.observe(duration);
		tracing::info!(
			runner_id=?self.runner_id,
			duration_seconds=duration,
			"fs creation completed",
		);

		Ok(())
	}

	pub async fn download_image(&self, ctx: &Ctx) -> Result<()> {
		let timer = Instant::now();

		ctx.image_download_handler
			.download(ctx, &self.config.image)
			.await?;

		let duration = timer.elapsed().as_secs_f64();
		crate::metrics::SETUP_DOWNLOAD_IMAGE_DURATION.observe(duration);

		Ok(())
	}

	pub(crate) async fn start_socket(self: &Arc<Self>, ctx: &Arc<Ctx>) -> Result<()> {
		tracing::info!(runner_id=?self.runner_id, "starting socket");

		let runner_path = ctx.runner_path(self.runner_id);
		let socket_path = runner_path.join("manager.sock");

		ensure!(
			socket_path.as_os_str().len() <= 104,
			"socket path ({}) length is too long (> 104 bytes aka SUN_LEN)",
			socket_path.display()
		);

		// Bind the listener (creates the socket file or reconnects if it already exists)
		let listener = UnixListener::bind(&socket_path).context("failed to bind unix listener")?;

		// Allow the container process to listen to the socket file
		fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o666))
			.await
			.context("failed to set permissions for socket file")?;

		// NOTE: This task and listener do not have to be cleaned up manually because they will stop when the
		// socket file is deleted (upon `cleanup_setup` call).
		let self2 = self.clone();
		let ctx2 = ctx.clone();
		tokio::task::spawn(async move {
			if let Err(err) = self2.handle_socket(&ctx2, listener).await {
				tracing::error!(runner_id=?self2.runner_id, ?err, "socket listener failed");

				if let Err(err) = self2.cleanup(&ctx2).await {
					tracing::error!(runner_id=?self2.runner_id, ?err, "cleanup failed");
				}
			}
		});

		Ok(())
	}

	async fn handle_socket(self: &Arc<Self>, ctx: &Arc<Ctx>, listener: UnixListener) -> Result<()> {
		let (stream, _) = listener.accept().await?;

		self.attach_socket(ctx, stream).await
	}

	pub async fn setup_oci_bundle(
		&self,
		ctx: &Ctx,
		ports: &protocol::HashableMap<String, protocol::ProxiedPort>,
	) -> Result<()> {
		let timer = Instant::now();
		tracing::info!(runner_id=?self.runner_id, "setting up oci bundle");

		let runner_path = ctx.runner_path(self.runner_id);
		let fs_path = runner_path.join("fs").join("upper");
		let netns_path = self.netns_path();
		let socket_path = runner_path.join("manager.sock");

		// Read the config.json from the user-provided OCI bundle
		tracing::info!(
			runner_id=?self.runner_id,
			"reading OCI bundle configuration",
		);
		let oci_bundle_config_path = fs_path.join("config.json");
		let user_config_json = fs::read_to_string(&oci_bundle_config_path)
			.await
			.context("failed to read oci config")?;
		let user_config =
			serde_json::from_str::<super::partial_oci_config::PartialOciConfig>(&user_config_json)?;

		// Build env
		tracing::info!(
			runner_id=?self.runner_id,
			"building environment variables",
		);
		let env = user_config
			.process
			.env
			.into_iter()
			.chain(
				self.build_default_env(ctx, &ports)
					.into_iter()
					.map(|(k, v)| format!("{k}={v}")),
			)
			.collect::<Vec<String>>();

		// Replace the config.json with a new config
		//
		// This config selectively uses parts from the user's OCI bundle in order to maintain security
		tracing::info!(
			runner_id=?self.runner_id,
			"generating OCI configuration",
		);
		let config = oci_config::config(oci_config::ConfigOpts {
			runner_path: &runner_path,
			netns_path: &netns_path,
			socket_path: &socket_path,
			args: user_config.process.args,
			env,
			user: user_config.process.user,
			cwd: user_config.process.cwd,
			use_resource_constraints: ctx.config().runner.use_resource_constraints(),
			cpu: self.config.resources.cpu,
			memory: self.config.resources.memory,
			memory_max: self.config.resources.memory_max,
		})?;
		// Parallelize file writes for better performance
		// Prepare content for all files before writing
		let config_json = serde_json::to_vec(&config)?;

		// resolv.conf content
		// See also rivet-actor.conflist in packages/services/cluster/src/workflows/server/install/install_scripts/files/pegboard_configure.sh
		let resolv_conf = indoc!(
			"
			nameserver 8.8.8.8
			nameserver 8.8.4.4
			nameserver 2001:4860:4860::8888
			nameserver 2001:4860:4860::8844
			options rotate
			options edns0
			options attempts:2
			"
		);

		// hosts file content
		let hosts_content = build_hosts_content(ctx);

		// Write all files in parallel
		tracing::info!(
			runner_id=?self.runner_id,
			"writing configuration files",
		);
		tokio::try_join!(
			fs::write(oci_bundle_config_path, config_json),
			fs::write(runner_path.join("resolv.conf"), resolv_conf),
			fs::write(runner_path.join("hosts"), hosts_content)
		)?;

		let duration = timer.elapsed().as_secs_f64();
		crate::metrics::SETUP_OCI_BUNDLE_DURATION.observe(duration);
		tracing::info!(
			runner_id=?self.runner_id,
			duration_seconds=duration,
			"OCI bundle setup completed"
		);

		Ok(())
	}

	// Only ran for bridge networking
	pub async fn setup_cni_network(
		&self,
		ctx: &Ctx,
		ports: &protocol::HashableMap<String, protocol::ProxiedPort>,
	) -> Result<()> {
		let timer = Instant::now();
		tracing::info!(runner_id=?self.runner_id, "setting up cni network");

		let runner_path = ctx.runner_path(self.runner_id);
		let netns_path = self.netns_path();

		tracing::info!(runner_id=?self.runner_id, "preparing cni port mappings");

		let cni_port_mappings = ports
			.iter()
			.map(|(_, port)| {
				json!({
					"HostPort": port.source,
					"ContainerPort": port.target,
					"Protocol": port.protocol.to_string(),
				})
			})
			.collect::<Vec<_>>();

		// MARK: Generate CNI parameters
		//
		// See https://github.com/containernetworking/cni/blob/b62753aa2bfa365c1ceaff6f25774a8047c896b5/cnitool/cnitool.go#L31

		// See Nomad capabilities equivalent:
		// https://github.com/hashicorp/nomad/blob/a8f0f2612ef9d283ed903721f8453a0c0c3f51c5/client/allocrunner/networking_cni.go#L105C46-L105C46
		//
		// See supported args:
		// https://github.com/containerd/go-cni/blob/6603d5bd8941d7f2026bb5627f6aa4ff434f859a/namespace_opts.go#L22
		tracing::info!(runner_id=?self.runner_id, "generating and writing cni parameters");
		let cni_params = json!({
			"portMappings": cni_port_mappings,
		});
		let cni_params_json = serde_json::to_string(&cni_params)?;
		fs::write(
			runner_path.join("cni-cap-args.json"),
			cni_params_json.as_bytes(),
		)
		.await?;

		// MARK: Create network
		//
		// See Nomad network creation:
		// https://github.com/hashicorp/nomad/blob/a8f0f2612ef9d283ed903721f8453a0c0c3f51c5/client/allocrunner/network_manager_linux.go#L119

		// Name of the network in /opt/cni/config/$NETWORK_NAME.conflist
		tracing::info!(runner_id=?self.runner_id, "creating network namespace");

		let cni_network_name = &ctx.config().cni.network_name();
		let cmd_out = Command::new("ip")
			.arg("netns")
			.arg("add")
			.arg(netns_path.file_name().context("bad netns path")?)
			.output()
			.await
			.context("failed to run `ip`")?;
		ensure!(
			cmd_out.status.success(),
			"failed `ip netns` command\n{}",
			std::str::from_utf8(&cmd_out.stderr)?
		);

		tracing::info!(
			runner_id=?self.runner_id,
			cni_network_name=cni_network_name,
			"adding network to namespace",
		);
		let cmd_out = Command::new("cnitool")
			.arg("add")
			.arg(cni_network_name)
			.arg(netns_path)
			.env("CNI_PATH", &ctx.config().cni.bin_path())
			.env("NETCONFPATH", &ctx.config().cni.config_path())
			.env("CNI_IFNAME", &ctx.config().cni.network_interface)
			.env("CAP_ARGS", cni_params_json)
			.output()
			.await
			.context("failed to run `cnitool`")?;
		ensure!(
			cmd_out.status.success(),
			"failed `cnitool` command\n{}",
			std::str::from_utf8(&cmd_out.stderr)?
		);

		let duration = timer.elapsed().as_secs_f64();
		crate::metrics::SETUP_CNI_NETWORK_DURATION.observe(duration);
		tracing::info!(
			runner_id=?self.runner_id,
			duration_seconds=duration,
			"cni network setup completed"
		);

		Ok(())
	}

	pub(crate) async fn bind_ports(
		&self,
		ctx: &Ctx,
	) -> Result<protocol::HashableMap<String, protocol::ProxiedPort>> {
		let timer = Instant::now();
		tracing::info!(runner_id=?self.runner_id, "binding ports");

		let (gg_ports, host_ports): (Vec<_>, Vec<_>) = self
			.config
			.ports
			.iter()
			.partition(|(_, port)| matches!(port.routing, protocol::PortRouting::GameGuard));

		tracing::info!(
			runner_id=?self.runner_id,
			gg_ports_count=gg_ports.len(),
			host_ports_count=host_ports.len(),
			"partitioned ports for binding"
		);

		// TODO: Could combine these into one
		let (gg_ports, host_ports) = tokio::try_join!(
			bind_ports_inner(
				ctx,
				self.runner_id,
				&gg_ports,
				ctx.config().network.lan_port_range_min()
					..=ctx.config().network.lan_port_range_max()
			),
			bind_ports_inner(
				ctx,
				self.runner_id,
				&host_ports,
				ctx.config().network.wan_port_range_min()
					..=ctx.config().network.wan_port_range_max()
			),
		)?;

		let proxied_ports = gg_ports
			.into_iter()
			.chain(host_ports.into_iter())
			.collect::<protocol::HashableMap<_, _>>();

		let duration = timer.elapsed().as_secs_f64();
		crate::metrics::SETUP_BIND_PORTS_DURATION.observe(duration);
		tracing::info!(
			runner_id=?self.runner_id,
			duration_seconds=duration,
			ports_count=proxied_ports.len(),
			"ports binding completed"
		);

		Ok(proxied_ports)
	}

	pub async fn spawn_orphaned(&self, ctx: &Ctx, env: &[(&str, String)]) -> Result<()> {
		{
			ensure!(
				self.pid
					.try_read()
					.context("pid should not be getting written to anywhere else")?
					.is_none(),
				"runner already has pid"
			);
		}

		// Prepare the arguments for the runner
		let runner_path = ctx.runner_path(self.runner_id);
		let runner_args = vec![runner_path.to_str().context("bad path")?, self.container_id()];

		// NOTE: Pipes are automatically closed on drop (OwnedFd)
		// Pipe communication between processes
		let (pipe_read, pipe_write) = pipe()?;

		// NOTE: This is why we fork the process twice: https://stackoverflow.com/a/5386753
		match unsafe { fork() }.context("process first fork failed")? {
			ForkResult::Parent { child } => {
				// Close the writing end of the pipe in the parent
				nix::unistd::close(pipe_write)?;

				// Ensure that the child process spawned successfully
				match waitpid(child, None).context("waitpid failed")? {
					WaitStatus::Exited(_, 0) => {
						// Read the second child's PID from the pipe
						let mut buf = [0u8; 4];
						read(pipe_read, &mut buf)?;
						let orphan_pid = Pid::from_raw(i32::from_le_bytes(buf));

						*self.pid.write().await = Some(orphan_pid);
						self.bump();

						tracing::info!(runner_id=?self.runner_id, pid=?orphan_pid, "runner spawned");

						// Update DB
						utils::sql::query(|| async {
							sqlx::query(indoc!(
								"
								UPDATE runners
								SET
									running_ts = ?2,
									pid = ?3
								WHERE
									runner_id = ?1
								",
							))
							.bind(self.runner_id)
							.bind(utils::now())
							.bind(orphan_pid.as_raw())
							.execute(&mut *ctx.sql().await?)
							.await
						})
						.await?;

						Ok(())
					}
					WaitStatus::Exited(_, status) => {
						bail!("Child process exited with status {}", status)
					}
					_ => bail!("Unexpected wait status for child process"),
				}
			}
			ForkResult::Child => {
				// Child process
				match unsafe { fork() } {
					Result::Ok(ForkResult::Parent { child }) => {
						// Write the second child's PID to the pipe
						let orphan_pid_bytes = child.as_raw().to_le_bytes();
						write(pipe_write, &orphan_pid_bytes)?;

						// Exit the intermediate child
						std::process::exit(0);
					}
					Result::Ok(ForkResult::Child) => {
						// Disassociate from the parent by creating a new session
						setsid().context("setsid failed")?;

						// Adjust nice, cpu priority, and OOM score
						let pid = std::process::id() as i32;
						utils::libc::set_nice_level(pid, 0).context("failed to set nice level")?;
						utils::libc::set_oom_score_adj(pid, 0)
							.context("failed to set oom score adjustment")?;
						utils::libc::set_scheduling_policy(
							pid,
							utils::libc::SchedPolicy::Other,
							// Must be 0 with SCHED_OTHER
							0,
						)
						.context("failed to set scheduling policy")?;

						// Exit immediately on fail in order to not leak process
						let err = std::process::Command::new("sh")
							.args(&runner_args)
							.envs(env.iter().cloned())
							.stdin(Stdio::null())
							.stdout(Stdio::null())
							.stderr(Stdio::null())
							.exec();
						eprintln!("exec failed: {err:?}");
						std::process::exit(1);
					}
					Err(err) => {
						// Exit immediately in order to not leak child process
						eprintln!("process second fork failed: {err:?}");
						std::process::exit(1);
					}
				}
			}
		}
	}

	// This function is meant to run gracefully-handled fallible steps to clean up every part of the setup
	// process
	#[tracing::instrument(skip_all)]
	pub async fn cleanup_setup(&self, ctx: &Ctx) {
		let runner_path = ctx.runner_path(self.runner_id);
		let netns_path = self.netns_path();

		// Clean up fs mounts
		if ctx.config().runner.use_mounts() {
			match Command::new("umount")
				.arg("-dl")
				.arg(runner_path.join("fs").join("upper"))
				.output()
				.await
			{
				Result::Ok(cmd_out) => {
					if !cmd_out.status.success() {
						tracing::error!(
							runner_id=?self.runner_id,
							stdout=%std::str::from_utf8(&cmd_out.stdout).unwrap_or("<failed to parse stdout>"),
							stderr=%std::str::from_utf8(&cmd_out.stderr).unwrap_or("<failed to parse stderr>"),
							"failed overlay `umount` command",
						);
					}
				}
				Err(err) => {
					tracing::error!(
						runner_id=?self.runner_id,
						?err,
						"failed to run overlay `umount` command",
					);
				}
			}

			match Command::new("umount")
				.arg("-dl")
				.arg(runner_path.join("fs"))
				.output()
				.await
			{
				Result::Ok(cmd_out) => {
					if !cmd_out.status.success() {
						tracing::error!(
							runner_id=?self.runner_id,
							stdout=%std::str::from_utf8(&cmd_out.stdout).unwrap_or("<failed to parse stdout>"),
							stderr=%std::str::from_utf8(&cmd_out.stderr).unwrap_or("<failed to parse stderr>"),
							"failed `umount` command",
						);
					}
				}
				Err(err) => {
					tracing::error!(
						runner_id=?self.runner_id,
						?err,
						"failed to run `umount` command",
					);
				}
			}
		}

		match self.config.image.kind {
			protocol::ImageKind::DockerImage | protocol::ImageKind::OciBundle => {
				// Clean up runc
				match Command::new("runc")
					.arg("delete")
					.arg("--force")
					.arg(self.container_id())
					.output()
					.await
				{
					Result::Ok(cmd_out) => {
						if !cmd_out.status.success() {
							tracing::error!(
								runner_id=?self.runner_id,
								stdout=%std::str::from_utf8(&cmd_out.stdout).unwrap_or("<failed to parse stdout>"),
								stderr=%std::str::from_utf8(&cmd_out.stderr).unwrap_or("<failed to parse stderr>"),
								"failed `runc` delete command",
							);
						}
					}
					Err(err) => {
						tracing::error!(
							runner_id=?self.runner_id,
							?err,
							"failed to run `runc` command",
						);
					}
				}

				if let protocol::NetworkMode::Bridge = self.config.network_mode {
					match fs::read_to_string(runner_path.join("cni-cap-args.json")).await {
						Result::Ok(cni_params_json) => {
							match Command::new("cnitool")
								.arg("del")
								.arg(&ctx.config().cni.network_name())
								.arg(&netns_path)
								.env("CNI_PATH", &ctx.config().cni.bin_path())
								.env("NETCONFPATH", &ctx.config().cni.config_path())
								.env("CNI_IFNAME", &ctx.config().cni.network_interface)
								.env("CAP_ARGS", cni_params_json)
								.output()
								.await
							{
								Result::Ok(cmd_out) => {
									if !cmd_out.status.success() {
										tracing::error!(
											runner_id=?self.runner_id,
											stdout=%std::str::from_utf8(&cmd_out.stdout).unwrap_or("<failed to parse stdout>"),
											stderr=%std::str::from_utf8(&cmd_out.stderr).unwrap_or("<failed to parse stderr>"),
											"failed `cnitool del` command",
										);
									}
								}
								Err(err) => {
									tracing::error!(
										runner_id=?self.runner_id,
										?err,
										"failed to run `cnitool` command",
									);
								}
							}
						}
						Err(err) => {
							tracing::error!(
								runner_id=?self.runner_id,
								?err,
								"failed to read `cni-cap-args.json`",
							);
						}
					}

					if let Some(netns_name) = netns_path.file_name() {
						// Clean up network
						match Command::new("ip")
							.arg("netns")
							.arg("del")
							.arg(netns_name)
							.output()
							.await
						{
							Result::Ok(cmd_out) => {
								if !cmd_out.status.success() {
									tracing::error!(
										runner_id=?self.runner_id,
										stdout=%std::str::from_utf8(&cmd_out.stdout).unwrap_or("<failed to parse stdout>"),
										stderr=%std::str::from_utf8(&cmd_out.stderr).unwrap_or("<failed to parse stderr>"),
										"failed `ip netns` command",
									);
								}
							}
							Err(err) => {
								tracing::error!(
									runner_id=?self.runner_id,
									?err,
									"failed to run `ip` command",
								);
							}
						}
					} else {
						tracing::error!(
							runner_id=?self.runner_id,
							?netns_path,
							"invalid netns path",
						);
					}
				}
			}
			protocol::ImageKind::JavaScript => {}
		}

		// Allow time for vector to pick up logs before they are deleted
		tokio::time::sleep(std::time::Duration::from_secs(5)).await;

		// Delete entire runner dir. Note that for actors using KV storage, it is persisted elsewhere and will
		// not be deleted by this (see `persist_storage` in the runner protocol).
		if let Err(err) = tokio::fs::remove_dir_all(&runner_path).await {
			tracing::error!(
				runner_id=?self.runner_id,
				?err,
				"failed to delete runner dir",
			);
		}
	}

	fn build_default_env(
		&self,
		ctx: &Ctx,
		ports: &protocol::HashableMap<String, protocol::ProxiedPort>,
	) -> HashMap<String, String> {
		self.config
			.env
			.iter()
			.map(|(k, v)| (k.clone(), v.clone()))
			// Add port env vars and api endpoint
			.chain(ports.iter().map(|(label, port)| {
				(
					format!("PORT_{}", label.to_uppercase().replace('-', "_")),
					port.target.to_string(),
				)
			}))
			.chain([
				(
					"RIVET_API_ENDPOINT".to_string(),
					ctx.config().cluster.api_endpoint.to_string(),
				),
				(
					"RIVET_MANAGER_SOCKET_PATH".to_string(),
					oci_config::socket_mount_dest_path()
						.to_str()
						.expect("invalid `socket_mount_dest_path`")
						.to_string(),
				),
			])
			.collect()
	}
}

fn build_hosts_content(ctx: &Ctx) -> String {
	let mut content = indoc!(
		"
		127.0.0.1	localhost
		::1			localhost ip6-localhost ip6-loopback
		"
	)
	.to_string();

	for host_entry in ctx.config().runner.custom_hosts() {
		content.push_str(&format!("{}\t{}\n", host_entry.ip, host_entry.hostname));
	}

	content
}

impl Runner {
	// Path to the created namespace
	fn netns_path(&self) -> PathBuf {
		if let protocol::NetworkMode::Host = self.config.network_mode {
			// Host network
			Path::new("/proc/1/ns/net").to_path_buf()
		} else {
			// CNI network that will be created
			Path::new("/var/run/netns").join(self.runner_id.to_string())
		}
	}
}

async fn bind_ports_inner(
	ctx: &Ctx,
	runner_id: Uuid,
	ports: &[(&String, &protocol::Port)],
	range: std::ops::RangeInclusive<u16>,
) -> Result<Vec<(String, protocol::ProxiedPort)>, Error> {
	if ports.is_empty() {
		return Ok(Vec::new());
	}

	// Compute the “modulus” for wrapping
	let truncated_max = (range.end() - range.start()) as i64;

	// Pick one random start‐offset for each protocol
	let mut tcp_cur = rand::thread_rng().gen_range(0..=truncated_max);
	let mut udp_cur = rand::thread_rng().gen_range(0..=truncated_max);

	let mut conn = ctx.sql().await?;
	let mut tx = conn.begin().await?;
	let mut bound = Vec::with_capacity(ports.len());

	for (label, port) in ports {
		let cur_offset = match port.protocol {
			protocol::TransportProtocol::Tcp => &mut tcp_cur,
			protocol::TransportProtocol::Udp => &mut udp_cur,
		};

		let row = sqlx::query_as::<_, (i64,)>(indoc!(
			"
			INSERT INTO runner_ports (runner_id, label, source, target, protocol)
			SELECT ?1, ?2, port, ?3, ?4
			FROM (
				WITH RECURSIVE
					nums(n, i) AS (
						SELECT ?5, ?5
						UNION ALL
						SELECT (n + 1) % (?6 + 1), i + 1
						FROM nums
						WHERE i < ?6 + ?7
					),
					available_ports(port) AS (
						SELECT nums.n + ?7
						FROM nums
						LEFT JOIN runner_ports AS p
						ON
							(nums.n + ?7) = p.source AND
							p.protocol = ?4 AND
							p.delete_ts IS NULL
						WHERE p.source IS NULL OR p.delete_ts IS NOT NULL
						LIMIT 1
					)
				SELECT port FROM available_ports
			)
			RETURNING source
			"
		))
		.bind(runner_id) // ?1
		.bind(label) // ?2
		.bind(port.target) // ?3
		.bind(port.protocol as i64) // ?4
		.bind(*cur_offset) // ?5: starting n for this protocol
		.bind(truncated_max) // ?6: modulus (range size)
		.bind(*range.start() as i64) // ?7: minimum port value
		.fetch_optional(&mut *tx)
		.await?;

		let Some((source,)) = row else {
			bail!("not enough available ports");
		};

		let host_port = source.try_into()?;

		bound.push((
			label.to_string(),
			protocol::ProxiedPort {
				source: host_port,
				// When no target port was selected, default to randomly selected host port
				target: port.target.unwrap_or(host_port),
				lan_hostname: ctx.config().network.lan_hostname.clone(),
				protocol: port.protocol,
			},
		));

		// bump the offset so next same‐protocol allocation starts from the next number
		*cur_offset = (*cur_offset + 1) % (truncated_max + 1);
	}

	tx.commit().await?;

	Ok(bound)
}
