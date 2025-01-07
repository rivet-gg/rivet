use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = ctx.op(faker::ops::user::Input {}).await.unwrap();
	let user_id = user_res.user_id;

	let email = util::faker::email();
    ctx.op(user::ops::identity::create::Input {
		user_id,
		identity: user::types::identity::Identity {
			kind: user::types::identity::Kind::Email(
				user::types::identity::Email {
					email: email.clone()
				}
			),
		},
	})
	.await
	.unwrap();

    ctx.op(user::ops::identity::delete::Input {
		user_ids: vec![user_id],
	})
	.await
	.unwrap();

	let (sql_exists,) = sqlx::query_as::<_, (bool,)>(
		"SELECT EXISTS (SELECT 1 FROM db_user_identity.emails WHERE email = $1)",
	)
	.bind(&email)
	.fetch_one(&ctx.crdb().await.unwrap())
	.await
	.unwrap();
	assert!(!sql_exists, "identity not deleted");
}
