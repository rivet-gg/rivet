use chirp_workflow::prelude::*;
use rand::Rng;

#[workflow_test]
async fn empty(ctx: TestCtx) {
	let res = ctx.op(::user::ops::get::Input {
		user_ids: Vec::new(),
	})
	.await
	.unwrap();
	assert!(res.users.is_empty());
}

#[workflow_test]
async fn fetch(ctx: TestCtx) {
	struct TestUser {
		user_id: Option<Uuid>,
		display_name: String,
		account_number: i64,
		bio: String,
	}

	// Generate test users
	let mut users = std::iter::repeat_with(|| TestUser {
		user_id: None,
		display_name: util::faker::display_name(),
		account_number: rand::thread_rng().gen_range(1..10000),
		bio: util::faker::ident(),
	})
	.take(8)
	.collect::<Vec<_>>();

	// Insert test users
	for user in &mut users {
		let user_res = ctx.op(faker::ops::user::Input {}).await.unwrap();
		let user_id = user_res.user_id;

		let mut update_sub = ctx.subscribe::<::user::workflows::user::Update>(
			("user_id", user_id)
		).await.unwrap();

		ctx.signal(::user::workflows::user::ProfileSet {
			display_name: Some(user.display_name.clone()),
			account_number: Some(user.account_number as u32),
			bio: Some(user.bio.clone()),
		})
		.tag("user_id", user_id)
		.send()
		.await.unwrap();

		update_sub.next().await.unwrap();

		user.user_id = Some(user_id);
	}

	// Fetch the users
	let res = ctx.op(::user::ops::get::Input {
		user_ids: users.iter().map(|u| u.user_id.unwrap()).collect(),
	})
	.await
	.unwrap();

	// Validate the users
	assert_eq!(users.len(), res.users.len());
	for user in &users {
		let user_res = res
			.users
			.iter()
			.find(|u| u.user_id.unwrap().as_uuid() == user.user_id.unwrap())
			.expect("user not returned");

		assert_eq!(user.display_name, user_res.display_name);
		assert_eq!(user.account_number, user_res.account_number as i64);
		assert_eq!(user.bio, user_res.bio);
	}
}
