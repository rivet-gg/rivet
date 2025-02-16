use chirp_workflow::prelude::*;

#[derive(Debug, Default)]
pub struct Input {}

#[derive(Debug)]
pub struct Output {
	pub user_id: Uuid,
}

#[operation]
pub async fn user(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output>  {
	let user_id = Uuid::new_v4();

    // TODO: Remove compat once ctx.workflow, ctx.subscribe exist on OpCtx
    let mut creation_sub = chirp_workflow::compat::subscribe::<
        user::workflows::user::CreateComplete, _
    >(&ctx.op_ctx(), ("user_id", user_id)).await?;

    chirp_workflow::compat::workflow(
        &ctx.op_ctx(),
        user::workflows::user::Input {
            user_id,
            display_name: None,
            is_already_in_db: false
        }
    )
    .await?
	.tag("user_id", user_id)
	.dispatch()
	.await?;

    creation_sub.next().await?;

	Ok(Output {
		user_id,
	})
}
