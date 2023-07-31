use chirp_worker::prelude::*;
use proto::backend;

#[worker_test]
async fn empty(ctx: TestCtx) {
	let game_res = op!([ctx] faker_game {
		..Default::default()
	})
	.await
	.unwrap();
	let game_id = game_res.game_id.unwrap();

	let res_a = op!([ctx] db_game_version_prepare {
		config: Some(backend::db::GameVersionConfig {
			database_name_id: "test".into(),
			schema: Some(backend::db::Schema::default())
		}),
		game_id: Some(game_id),
	})
	.await
	.unwrap();

	let res_b = op!([ctx] db_game_version_prepare {
		config: Some(backend::db::GameVersionConfig {
			database_name_id: "test".into(),
			schema: Some(backend::db::Schema::default())
		}),
		game_id: Some(game_id),
	})
	.await
	.unwrap();

	assert_eq!(
		res_a.config_ctx.as_ref().unwrap().database_id,
		res_b.config_ctx.as_ref().unwrap().database_id,
		"did not reuse database"
	);
}
