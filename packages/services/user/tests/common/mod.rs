use chirp_workflow::prelude::*;

// TODO: Move back to faker op after ops can dispatch workflows
#[derive(Debug)]
pub struct FakerUserOutput {
    pub user_id: Uuid,
}

pub async fn make_test_user(ctx: &TestCtx) -> GlobalResult<FakerUserOutput> {
    let user_id = Uuid::new_v4();

    tracing::debug!(%user_id, "creating test user wf");

    let mut creation_sub = ctx
        .subscribe::<user::workflows::user::CreateComplete>(("user_id", user_id))
        .await?;

    ctx.workflow(user::workflows::user::Input {
		user_id,
		display_name: None,
	})
	.tag("user_id", user_id)
	.dispatch()
    .await?;

    creation_sub.next().await?;

    Ok(FakerUserOutput {
        user_id
    })
}