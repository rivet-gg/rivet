use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = ctx.op(faker::ops::user::Input {}).await.unwrap();
	let user_id = user_res.user_id;

	let res = ctx.op(user::ops::token_create::Input {
		user_id,
		client: backend::net::ClientInfo::default()
	})
	.await
	.unwrap();

	assert!(res.token.starts_with("usr"));
	assert!(res.refresh_token.starts_with("usr_rf"));
}
