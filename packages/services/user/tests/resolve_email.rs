use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

mod common;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = common::make_test_user(&ctx).await.unwrap();
	let user_id = user_res.user_id;

	let email = util::faker::email();
	ctx.op(::user::ops::identity::create::Input {
		user_id: user_id,
		identity: backend::user_identity::Identity {
			kind: Some(backend::user_identity::identity::Kind::Email(
				backend::user_identity::identity::Email {
					email: email.clone()
				}
			)),
		},
	})
	.await
	.unwrap();

	let res = ctx.op(::user::ops::resolve_email::Input {
		emails: vec![email.clone(), util::faker::email()],
	})
	.await
	.unwrap();
	assert_eq!(1, res.users.len());
	let user = res.users.first().unwrap();
	assert_eq!(user_id, user.user_id);
}
