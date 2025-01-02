use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = op!([ctx] faker_user {}).await.unwrap();
	let user_id = user_res.user_id.as_ref().unwrap().as_uuid();

	let res = ctx.op(::user::ops::token_create::Input {
		user_id,
		client: backend::net::ClientInfo::default()
	})
	.await
	.unwrap();

	assert!(res.token.starts_with("usr"));
	assert!(res.refresh_token.starts_with("usr_rf"));
}
