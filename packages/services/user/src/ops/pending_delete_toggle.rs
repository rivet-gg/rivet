use chirp_workflow::prelude::*;
use proto::backend::{self, pkg::*};
use rivet_operation::prelude::proto;

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
	let identity = op!([ctx] user_identity_get {
		user_ids: vec![user_id.into()],
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

	msg!([ctx] user::msg::update(user_id) {
		user_id: Some(user_id.into()),
	})
	.await?;

	Ok(Output {})
}