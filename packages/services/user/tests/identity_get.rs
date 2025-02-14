use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = ctx.op(faker::ops::user::Input {}).await.unwrap();
	let user_id = user_res.user_id;

	let email = util::faker::email();
    ctx.op(user::ops::identity::create::Input {
		user_id,
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

	let res = ctx.op(user::ops::identity::get::Input {
		user_ids: vec![user_id, Uuid::new_v4()],
	})
	.await
	.unwrap();
	assert_eq!(1, res.users.len());
	assert_eq!(
		1,
		res.users
			.iter()
			.find(|u| u.user_id == user_id)
			.unwrap()
			.identities
			.len()
	);
}
