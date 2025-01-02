use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let user_res = op!([ctx] faker_user {}).await.unwrap();
	let user_id = user_res.user_id.as_ref().unwrap().as_uuid();

	// Register user
	let email = util::faker::email();
	let _res = ctx.op(::user::ops::identity::create::Input {
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

	ctx.op(::user::ops::pending_delete_toggle::Input {
		user_id,
		active: true,
	})
	.await
	.unwrap();

	let (delete_request_ts,): (Option<i64>,) = sqlx::query_as(indoc!(
		"
		SELECT delete_request_ts
			FROM db_user.users
			WHERE
				user_id = $1
		",
	))
	.bind(user_id)
	.fetch_one(&ctx.crdb().await.unwrap())
	.await
	.unwrap();

	assert!(delete_request_ts.is_some());

	ctx.op(::user::ops::pending_delete_toggle::Input {
		user_id,
		active: false,
	})
	.await
	.unwrap();

	let (delete_request_ts,): (Option<i64>,) = sqlx::query_as(indoc!(
		"
		SELECT delete_request_ts
			FROM db_user.users
			WHERE
				user_id = $1
		",
	))
	.bind(user_id)
	.fetch_one(&ctx.crdb().await.unwrap())
	.await
	.unwrap();

	assert!(delete_request_ts.is_none());
}
