use chirp_worker::prelude::*;
use proto::backend::pkg::*;

#[worker_test]
async fn create(ctx: TestCtx) {
	let database_id = Uuid::new_v4();
	msg!([ctx] db::msg::create(database_id) -> db::msg::create_complete {
		database_id: Some(database_id.into()),
		owner_team_id: Some(Uuid::new_v4().into()),
		name_id: "test".into(),
	})
	.await
	.unwrap();

	let row =
		sqlx::query_as::<_, (Uuid,)>("SELECT database_id FROM databases WHERE database_id = $1")
			.bind(database_id)
			.fetch_optional(&ctx.crdb("db-db").await.unwrap())
			.await
			.unwrap();
	assert!(row.is_some());
}
