use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker_test]
async fn schema_apply(ctx: TestCtx) {
	let database_id = Uuid::new_v4();
	msg!([ctx] db::msg::create(database_id) -> db::msg::create_complete {
		database_id: Some(database_id.into()),
		owner_team_id: Some(Uuid::new_v4().into()),
		name_id: "test".into(),
	})
	.await
	.unwrap();

	msg!([ctx] db::msg::schema_apply(database_id) -> Result<db::msg::schema_apply_complete, db::msg::schema_apply_fail> {
		database_id: Some(database_id.into()),
		schema: Some(backend::db::Schema {
			collections: vec![
				backend::db::Collection {
					name_id: "test".into(),
					fields: vec![
						backend::db::Field {
							name_id: "test".into(),
							r#type: backend::db::field::Type::String.into(),
							optional: false,
						},
					],
				},
			],
		}),
	})
	.await
	.unwrap();
}
