use anyhow::Result;
use duct::cmd;
use tokio::task::block_in_place;

pub struct Credentials<'a> {
	pub hostname: &'a str,
	pub port: u16,
	pub username: &'a str,
	pub password: Option<&'a str>,
	pub db_name: &'a str,
}

#[derive(PartialEq)]
pub enum Compatability {
	Native,
	Cockroach,
}

pub async fn exec<'a>(
	creds: Credentials<'a>,
	compat: Compatability,
	query: Option<&'a str>,
) -> Result<()> {
	if let Some(query) = query {
		block_in_place(|| {
			let mut cmd = cmd!(
				"psql",
				"-h",
				creds.hostname,
				"-p",
				creds.port.to_string(),
				"-U",
				creds.username,
				creds.db_name,
				"-c",
				query,
			);
			if let Some(password) = creds.password {
				cmd = cmd.env("PGPASSWORD", password);
			}
			if compat == Compatability::Cockroach {
				cmd = cmd.env("PGCLIENTENCODING", "utf-8");
			}
			cmd.run()
		})?;
	} else {
		block_in_place(|| {
			let mut cmd = cmd!(
				"psql",
				"-h",
				creds.hostname,
				"-p",
				creds.port.to_string(),
				"-U",
				creds.username,
				creds.db_name
			);
			if let Some(password) = creds.password {
				cmd = cmd.env("PGPASSWORD", password);
			}
			if compat == Compatability::Cockroach {
				cmd = cmd.env("PGCLIENTENCODING", "utf-8");
			}
			cmd.run()
		})?;
	}

	Ok(())
}
