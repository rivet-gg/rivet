use chirp_workflow::prelude::*;

pub async fn run(ctx: &mut WorkflowCtx) -> GlobalResult<()> {
	ctx.activity(MigrateInitInput {}).await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct MigrateInitInput {}

#[activity(MigrateInit)]
async fn migrate_init(ctx: &ActivityCtx, _input: &MigrateInitInput) -> GlobalResult<()> {
	let pool = ctx.sqlite().await?;
		
	sql_execute!(
		[ctx, pool]
		"
		CREATE TABLE test (
		
		)
		",
	)
	.await?;

	Ok(())
}
