use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn basic(ctx: TestCtx) {
	let database_id = Uuid::new_v4();
	msg!([ctx] db::msg::create(database_id) -> db::msg::create_complete {
		database_id: Some(database_id.into()),
		owner_team_id: Some(Uuid::new_v4().into()),
		name_id: "test".into(),
	})
	.await
	.unwrap();

	let db_res = op!([ctx] db_get {
		database_ids: vec![database_id.into()],
	})
	.await
	.unwrap();
	assert_eq!(1, db_res.databases.len());
}
