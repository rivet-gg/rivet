use anyhow::Result;
use clap::Parser;
use std::result::Result as StdResult;

#[derive(Parser)]
pub struct Opts {
	#[clap(long, short)]
	/// Output in JSON format
	json: bool,
}

impl Opts {
	pub async fn execute(&self) -> Result<()> {
		let ctx = crate::util::login::load_or_login().await?;
		match self.json {
			true => {
				match serde_json::to_string(&ctx.project.namespaces) {
					StdResult::Ok(json) => println!("{}", json),
					StdResult::Err(err) => eprintln!("failed to serialize output: {err:?}"),
				}
				Ok(())
			}
			false => {
				for ns in &ctx.project.namespaces {
					println!("{} ({})", ns.display_name, ns.name_id);
				}
				Ok(())
			}
		}
	}
}
