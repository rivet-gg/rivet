use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto;
use proto::backend::{pkg::*};
use serde_json::json;

#[derive(Debug)]
pub struct Input {
    pub user_id: Uuid,
    pub upload_id: Uuid,
}

#[derive(Debug)]
pub struct Output {}


#[operation]
pub async fn avatar_upload_complete(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	let user_id = input.user_id;

	op!([ctx] upload_complete {
		upload_id: Some(input.upload_id.into()),
		bucket: Some("bucket-user-avatar".into()),
	})
	.await?;

	// Set avatar id
	sql_execute!(
		[ctx]
		"
		UPDATE db_user.users set profile_id = $2
		WHERE user_id = $1
		",
		user_id,
		input.upload_id,
	)
	.await?;

	ctx.cache().purge("user", [user_id]).await?;

	msg!([ctx] user::msg::update(user_id) {
		user_id: Some(user_id.into()),
	})
	.await?;

	msg!([ctx] analytics::msg::event_create() {
		events: vec![
			analytics::msg::event_create::Event {
				event_id: Some(Uuid::new_v4().into()),
				name: "user.avatar_set".into(),
				properties_json: Some(serde_json::to_string(&json!({
					"user_id": user_id,
				}))?),
				..Default::default()
			}
		],
	})
	.await?;

	Ok(Output {})
}