use anyhow::Result;
use duct::cmd;
use tokio::task::block_in_place;

pub struct Credentials<'a> {
	pub hostname: &'a str,
	pub port: u16,
	pub username: &'a str,
	pub password: Option<&'a str>,
	pub keyspace: &'a str,
}

pub async fn exec<'a>(creds: Credentials<'a>, query: Option<&'a str>) -> Result<()> {
	let password = creds.password.unwrap_or("");
	if let Some(query) = query {
		block_in_place(|| {
			cmd!(
				"docker",
				"run",
				"-it",
				"--network",
				"host",
				"--entrypoint",
				"/bin/cqlsh",
				"scylladb/scylla",
				creds.hostname,
				creds.port.to_string(),
				"-k",
				creds.keyspace,
				"-u",
				creds.username,
				"-p",
				password,
				"-e",
				query
			)
			.run()
		})?;
	} else {
		block_in_place(|| {
			cmd!(
				"docker",
				"run",
				"-it",
				"--network",
				"host",
				"--entrypoint",
				"/bin/cqlsh",
				"scylladb/scylla",
				creds.hostname,
				creds.port.to_string(),
				"-k",
				creds.keyspace,
				"-u",
				creds.username,
				"-p",
				password,
			)
			.run()
		})?;
	}

	Ok(())
}
