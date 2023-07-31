use chirp_worker::prelude::*;
use proto::backend::{self, db::*};

#[worker_test]
async fn empty(ctx: TestCtx) {
	let game_res = op!([ctx] faker_game {
		..Default::default()
	})
	.await
	.unwrap();

	let version_id = Into::<common::Uuid>::into(Uuid::new_v4());

	let config = backend::db::GameVersionConfig {
		database_name_id: "test".into(),
		schema: Some(backend::db::Schema::default()),
	};

	let prepare_res = op!([ctx] db_game_version_prepare {
		config: Some(config.clone()),
		game_id: game_res.game_id.clone(),
	})
	.await
	.unwrap();

	op!([ctx] db_game_version_publish {
		version_id: Some(version_id),
		config: Some(config),
		config_ctx: Some(prepare_res.config_ctx.clone().unwrap()),
	})
	.await
	.unwrap();

	let res = op!([ctx] db_game_version_get {
		version_ids: vec![version_id]
	})
	.await
	.unwrap();
	res.versions.first().expect("version not found");
}
