use std::{path::Path, str::FromStr};

use anyhow::*;
use serde_json::json;
use uuid::Uuid;

pub struct ShellQuery {
	pub svc: String,
	pub query: Option<String>,
}

pub struct ShellContext<'a> {
	pub queries: &'a [ShellQuery],
}

pub async fn redis_shell(config: rivet_config::Config, shell_ctx: ShellContext<'_>) -> Result<()> {
	let ShellContext { queries, .. } = shell_ctx;

	for ShellQuery { svc, query } in queries {
		let server_config = config.server.as_ref().context("missing server")?;
		let redis_config = match svc.as_str() {
			"ephemeral" => &server_config.redis.ephemeral,
			"persistent" => &server_config.redis.persistent,
			_ => bail!("redis svc can only be ephemeral or persistent"),
		};

		tracing::info!(?svc, "connecting to redis");

		if query.is_some() {
			bail!("cannot pass query to redis shell at the moment");
		}

		let parsed_url = redis_config.url.clone();
		let hostname = parsed_url.host_str().context("missing hostname")?;
		let port = parsed_url.port().unwrap_or(6379);

		let mut cmd = std::process::Command::new("redis-cli");
		cmd.args(["-h", hostname, "-p", &port.to_string(), "-c", "--tls"]);
		if let Some(username) = &redis_config.username {
			cmd.arg("--user").arg(username);
		}
		if let Some(password) = &redis_config.password {
			cmd.arg("--pass").arg(password.read());
		}

		let ca_path = format!("/usr/local/share/ca-certificates/redis-{svc}-ca.crt");
		if Path::new(&ca_path).exists() {
			cmd.arg("--cacert").arg(&ca_path);
		}

		if let Some(password) = parsed_url.password() {
			cmd.env("REDISCLI_AUTH", password);
		}

		cmd.status()?;
	}

	Ok(())
}

pub async fn cockroachdb_shell(
	config: rivet_config::Config,
	shell_ctx: ShellContext<'_>,
) -> Result<()> {
	let ShellContext { queries, .. } = shell_ctx;

	let server_config = config.server.as_ref().context("server not enabled")?;

	tracing::info!("connecting to cockroachdb");

	// Combine all queries into one command
	for ShellQuery {
		svc: db_name,
		query,
	} in queries
	{
		let mut parsed_url = server_config.cockroachdb.url.clone();
		parsed_url.set_path(&format!("/{}", db_name));

		let ca_path = "/usr/local/share/ca-certificates/crdb-ca.crt";
		if Path::new(&ca_path).exists() {
			parsed_url.set_query(Some(&format!("sslmode=verify-ca&sslrootcert={ca_path}")));
		} else {
			parsed_url.set_query(None);
		}
		parsed_url
			.set_username(&server_config.cockroachdb.username)
			.ok()
			.context("failed to set username")?;
		if let Some(password) = &server_config.cockroachdb.password {
			parsed_url
				.set_password(Some(password.read()))
				.ok()
				.context("failed to set password")?;
		}

		let db_url = parsed_url.to_string();

		let mut cmd = std::process::Command::new("psql");
		cmd.arg(&db_url);

		if let Some(query) = query {
			cmd.args(["-c", query]);
		}

		// See https://github.com/cockroachdb/cockroach/issues/37129#issuecomment-600115995
		cmd.env("PGCLIENTENCODING", "utf-8");

		cmd.status()?;
	}

	Ok(())
}

pub async fn clickhouse_shell(
	config: rivet_config::Config,
	shell_ctx: ShellContext<'_>,
) -> Result<()> {
	let ShellContext { queries, .. } = shell_ctx;

	let server_config = config.server.as_ref().context("server not enabled")?;

	tracing::info!("connecting to clickhouse");

	for ShellQuery {
		svc: db_name,
		query,
	} in queries
	{
		let clickhouse_config = server_config
			.clickhouse
			.as_ref()
			.context("clickhouse disabled")?;
		let parsed_url = clickhouse_config.native_url.clone();

		let hostname = parsed_url.host_str().unwrap_or("localhost");
		let port = parsed_url.port().unwrap_or(9440).to_string();

		let ca_path = "/usr/local/share/ca-certificates/clickhouse-ca.crt";
		let config = json!({
			"user": clickhouse_config.username,
			"password": clickhouse_config.password.as_ref().map(|x| x.read().clone()),
			"openSSL": if Path::new(&ca_path).exists() {
				json!({
					"client": {
						"caConfig": ca_path
					}
				})
			} else {
				json!(null)
			}
		});

		let mut config_file = tempfile::Builder::new().suffix(".yaml").tempfile()?;
		serde_yaml::to_writer(&mut config_file, &config)?;

		let mut cmd = std::process::Command::new("clickhouse-client");
		cmd.arg("--host")
			.arg(hostname)
			.arg("--port")
			.arg(&port)
			.arg("--user")
			.arg(&clickhouse_config.username)
			.arg("--database")
			.arg(db_name)
			.arg("--config-file")
			.arg(config_file.path());
		if let Some(password) = &clickhouse_config.password {
			cmd.arg("--password").arg(password.read());
		}

		if let Some(query) = query {
			cmd.arg("--multiquery").arg(query);
		}

		cmd.status()?;
	}

	Ok(())
}

pub async fn wf_sqlite_shell(
	config: rivet_config::Config,
	shell_ctx: ShellContext<'_>,
	internal: bool,
) -> Result<()> {
	let ShellContext { queries, .. } = shell_ctx;

	let pools = rivet_pools::Pools::new(config.clone()).await?;

	// Combine all queries into one command
	for ShellQuery {
		svc: workflow_id,
		query,
	} in queries
	{
		let workflow_id = Uuid::from_str(workflow_id).context("could not parse input as UUID")?;
		let key = if internal {
			chirp_workflow::db::sqlite_db_name_internal(workflow_id)
		} else {
			chirp_workflow::db::sqlite_db_name_data(workflow_id)
		};

		rivet_term::status::warn(
			"WARNING",
			"Database opened in WRITE mode. Modifications made will only be committed after the shell closes. This may cause changes made outside of this shell to be overwritten."
		);
		println!();

		tracing::info!(?key, "connecting to sqlite");
		let pool = pools.sqlite(key, false).await?;

		// Close the pool since we are going to be using the CLI
		pool.close().await;

		let mut cmd = std::process::Command::new("/root/go/bin/usql");
		cmd.arg(format!("sqlite:{}", pool.db_path().display()));

		if let Some(query) = query {
			cmd.args(["-c", query]);
		}

		cmd.status().context("failed running usql")?;

		rivet_term::status::progress("Evicting database", "");
		pool.evict().await.map_err(|x| anyhow!("{x}"))?;
		rivet_term::status::success("Evicted", "");
	}

	Ok(())
}
