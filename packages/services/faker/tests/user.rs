use chirp_workflow::prelude::*;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let res = ctx.op(faker::ops::user::Input {}).await.unwrap();

	let get_res = ctx.op(::user::ops::get::Input {
		user_ids: vec![res.user_id],
	})
	.await
	.unwrap();

	assert_eq!(1, get_res.users.len(), "user not created");
}
