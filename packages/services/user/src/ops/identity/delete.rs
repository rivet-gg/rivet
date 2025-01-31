use chirp_workflow::prelude::*;

#[derive(Debug)]
pub struct Input {
    pub user_ids: Vec<Uuid>
}

#[derive(Debug)]
pub struct Output {}


#[operation]
pub async fn create(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	sql_execute!(
		[ctx]
		"
		DELETE FROM db_user_identity.emails
		WHERE user_id = ANY($1)
		",
		&input.user_ids,
	)
	.await?;

	ctx.cache()
		.purge("user_identity.identity", input.user_ids.clone())
		.await?;

	Ok(Output {})
}