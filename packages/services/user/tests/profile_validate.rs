use chirp_workflow::prelude::*;

mod common;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = common::make_test_user(&ctx).await.unwrap();
	let user_id = user_res.user_id;

	let res = ctx.op(::user::ops::profile_validate::Input {
		user_id: user_id,
		display_name: Some("  bad display name".to_owned()),
		account_number: Some(10000),
		bio: Some("bad\n\n\n\n\n\nbio".to_owned())
	})
	.await
	.unwrap();

	assert_eq!(res.errors.len(), 3, "validation failed");
}
