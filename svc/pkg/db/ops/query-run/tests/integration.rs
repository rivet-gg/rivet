use chirp_worker::prelude::*;
use proto::backend::{
	self,
	db::{field::Type as FT, value::Type as VT, Field, Value},
	pkg::*,
};
use std::collections::HashMap;

#[worker_test]
async fn basic(ctx: TestCtx) {
	// Create database
	let database_id = Uuid::new_v4();
	msg!([ctx] db::msg::create(database_id) -> db::msg::create_complete {
		database_id: Some(database_id.into()),
		owner_team_id: Some(Uuid::new_v4().into()),
		name_id: "test".into(),
	})
	.await
	.unwrap();

	// Apply schema
	msg!([ctx] db::msg::schema_apply(database_id) -> Result<db::msg::schema_apply_complete, db::msg::schema_apply_fail> {
		database_id: Some(database_id.into()),
		schema: Some(backend::db::Schema {
			collections: vec![
				backend::db::Collection {
					name_id: "test".into(),
					fields: vec![
						Field {
							name_id: "my_int".into(),
							r#type: FT::Int.into(),
							optional: false,
						},
						Field {
							name_id: "my_float".into(),
							r#type: FT::Float.into(),
							optional: false,
						},
						Field {
							name_id: "my_bool".into(),
							r#type: FT::Bool.into(),
							optional: false,
						},
						Field {
							name_id: "my_string".into(),
							r#type: FT::String.into(),
							optional: false,
						},
					],
				},
			],
		}),
	})
	.await
	.unwrap();

	// Run query
	let fields = {
		let mut x = HashMap::new();
		x.insert(
			"my_int".into(),
			Value {
				r#type: Some(VT::Int(42)),
			},
		);
		x.insert(
			"my_float".into(),
			Value {
				r#type: Some(VT::Float(4.2)),
			},
		);
		x.insert(
			"my_bool".into(),
			Value {
				r#type: Some(VT::Bool(true)),
			},
		);
		x.insert(
			"my_string".into(),
			Value {
				r#type: Some(VT::String("hello, world!".into())),
			},
		);
		x
	};
	op!([ctx] db_query_run {
		database_id: Some(database_id.into()),
		query: Some(backend::db::Query {
			kind: Some(backend::db::query::Kind::Insert(backend::db::query::Insert {
				collection: "test".into(),
				fields: fields,
			})),
		}),
	})
	.await
	.unwrap();
}
