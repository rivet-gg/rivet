use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

use game_user::create::Response as GameUser;

async fn create_game_users(
	ctx: &TestCtx,
	count: usize,
	namespace_id: Option<Uuid>,
) -> (Uuid, Vec<GameUser>) {
	let mut users = vec![];

	let namespace_id = namespace_id.unwrap_or_else(Uuid::new_v4);

	println!("Creating {count} game users...");

	for i in 0..count {
		println!("Creating game user {i}...");

		let operation = op!([ctx] game_user_create {
			namespace_id: Some(namespace_id.into()),
			user_id: Some(Uuid::new_v4().into())
		});

		users.push(operation.await.expect("Error creating game user!"));

		println!("Game user {i} created.");
	}
	println!("{count} game users created.");

	(namespace_id, users)
}

#[worker_test]
async fn test_return_ok(ctx: TestCtx) {
	op!([ctx] game_user_recommend { count: 0 }).await.unwrap();
}

#[worker_test]
async fn check_order(ctx: TestCtx) {
	let generated_users = create_game_users(&ctx, 10, None).await.1;

	let mut generated_user_ids = generated_users
		.into_iter()
		.map(|game_user| {
			game_user
				.game_user_id
				.map(Uuid::from)
				.ok_or("No game_user_id found in the game user.")
		})
		.collect::<Result<Vec<Uuid>, _>>()
		.unwrap();

	// So we don't have to worry about accidental mutability.
	let generated_user_ids = {
		let mut temp_gen_usr_ids = generated_user_ids;
		temp_gen_usr_ids.reverse();
		temp_gen_usr_ids
	};

	let returned_user_ids: Vec<_> = op!([ctx] game_user_recommend { count: 10 })
		.await
		.unwrap()
		.game_user_ids;
	let returned_user_ids: Vec<_> = returned_user_ids.into_iter().map(Uuid::from).collect();

	assert_eq!(generated_user_ids, returned_user_ids);
}

#[worker_test]
async fn check_count(ctx: TestCtx) {
	let namespace = Uuid::new_v4();

	const MAX_USERS: usize = 10;

	create_game_users(&ctx, MAX_USERS, None).await;

	for target_len in 0..MAX_USERS {
		println!("Requesting {target_len} game users...");

		let count = target_len as u32;

		let len = op!([ctx] game_user_recommend { count: count })
			.await
			.unwrap()
			.game_user_ids
			.len();

		println!("Returned {len} game user ids.");
		assert_eq!(len, target_len);
	}
	println!("Requests for 0 through {MAX_USERS} game user ids successful.");
}
