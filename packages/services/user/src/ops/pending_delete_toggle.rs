use chirp_workflow::prelude::*;

#[derive(Debug)]
pub struct Input {
    pub user_id: Uuid,
    pub active: bool
}

#[derive(Debug)]
pub struct Output {
}


#[operation]
pub async fn pending_delete_toggle(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	let user_id = input.user_id;

	// Verify the user is registered
	let identity = ctx.op(crate::ops::identity::get::Input {
		user_ids: vec![user_id]
	})
	.await?;
	let identities = &unwrap_ref!(identity.users.first()).identities;
	ensure_with!(!identities.is_empty(), IDENTITY_NOT_REGISTERED);

	sql_execute!(
		[ctx]
		"UPDATE db_user.users SET delete_request_ts = $2 WHERE user_id = $1",
		user_id,
		input.active.then(util::timestamp::now),
	)
	.await?;

	ctx.cache().purge("user", [user_id]).await?;

	chirp_workflow::compat::signal(
		ctx.op_ctx(),
		crate::workflows::user::ToggledPendingDeletion {}
	).await?.tag("user_id", user_id).send().await?;

	Ok(Output {})
}