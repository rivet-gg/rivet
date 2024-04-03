use anyhow::*;
use bolt_core::{context::ProjectContext, tasks, utils};
use clap::Parser;

#[derive(Parser)]
pub enum SubCommand {
	/// Generates a login link for the given access token. Automatically turns the existing user into an
	/// admin (or creates a new admin if no user).
	Login {
		#[clap(default_value = "root")]
		name: String,
	},
	/// Cluster related operations
	Cluster {
		#[clap(subcommand)]
		sub: ClusterSubCommand,
	},
}

#[derive(Parser)]
pub enum ClusterSubCommand {
    /// Creates a new cluster
    Create {
        /// The name of the cluster
        #[clap(short, long)]
        name: String,
        /// The ID of the owner team
        #[clap(short, long)]
        owner_team_id: String,
    },
    /// Deletes an existing cluster
    Delete,
}

impl SubCommand {
	pub async fn execute(self, ctx: ProjectContext) -> Result<()> {
		match self {
			Self::Login { name } => {
				assert!(
					ctx.ns().rivet.login.enable_admin,
					"admin login is not enabled in the namespace config (rivet.admin_login)"
				);

				tasks::api::access_token_login(&ctx, name).await?;

				utils::ringadingding();

				Ok(())
			},
            Self::Cluster { sub } => {
                match sub {
                    ClusterSubCommand::Create { name, owner_team_id } => {
                        // Handle cluster creation here
                        // You can now use `name` and `owner_team_id`
                    },
                    ClusterSubCommand::Delete => {
                        // Handle cluster deletion here
                    },
                }
                Ok(())
            }
		}
	}
}