use chirp_workflow::prelude::*;

use super::Input;

pub fn run(ctx: &mut WorkflowCtx, input: &Input) -> GlobalResult<()> {
	ctx.activity(MigrateInitInput {}).await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct MigrateInitInput {}

#[activity(MigrateInit)]
async fn migrate_init(ctx: &ActivityCtx, &MigrateInitInput) -> GlobalResult<()> {
	sql_execute!(
		[ctx]
		"
		CREATE TABLE test (
		
		)
		",
	)
	.await
	.map_err(Into::into)
}
