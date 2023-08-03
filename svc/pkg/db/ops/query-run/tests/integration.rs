use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};
use serde_json::json;

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
					entry_schema: r#"{"type":"object", "properties":{}}"#.into(),
					indexes: vec![],
				},
			],
		}),
	})
	.await
	.unwrap().unwrap();

	// Insert entry
	let entry_id = {
		let entry = json!({
			"my_int": 42,
			"my_float": 3.14,
			"my_string": "hello, world!",
			"my_bool": true,
			"my_object": {
				"foo": "bar",
				"baz": 42,
			},
		});
		let res = op!([ctx] db_query_run {
			database_id: Some(database_id.into()),
			query: Some(backend::db::Query {
				kind: Some(backend::db::query::Kind::Insert(backend::db::query::Insert {
					collection: "test".into(),
					entries: vec![backend::db::query::insert::Entry {
						value: serde_json::to_string(&entry).unwrap(),
					}]
				})),
			}),
		})
		.await
		.unwrap();
		let id = res.entry_ids.first().unwrap().clone();

		id
	};

	// Get entry
	{
		let res = op!([ctx] db_query_run {
			database_id: Some(database_id.into()),
			query: Some(backend::db::Query {
				kind: Some(backend::db::query::Kind::Get(backend::db::query::Get {
					collection: "test".into(),
					entry_ids: vec![entry_id],
				})),
			}),
		})
		.await
		.unwrap();
		assert_eq!(1, res.entries.len());
	}

	// // Update entry by ID
	// {
	// 	let insert_set = {
	// 		let mut x = HashMap::new();
	// 		x.insert(
	// 			"my_int".into(),
	// 			Value {
	// 				r#type: Some(VT::Int(123)),
	// 			},
	// 		);
	// 		x
	// 	};
	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Update(backend::db::query::Update {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "_id".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::String(entry_id.clone())) })),
	// 					}
	// 				],
	// 				set: insert_set
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	assert_eq!(1, res.entries_affected);

	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Fetch(backend::db::query::Fetch {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "_id".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::String(entry_id.clone())) })),
	// 					}
	// 				]
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	let fetched_entry = res.fetched_entries.first().unwrap();
	// 	assert_eq!(
	// 		VT::Int(123),
	// 		fetched_entry
	// 			.entry
	// 			.get("my_int")
	// 			.unwrap()
	// 			.r#type
	// 			.clone()
	// 			.unwrap()
	// 	);
	// }

	// // Update entry by user defined field
	// {
	// 	let insert_set = {
	// 		let mut x = HashMap::new();
	// 		x.insert(
	// 			"my_string".into(),
	// 			Value {
	// 				r#type: Some(VT::String("foo bar".into())),
	// 			},
	// 		);
	// 		x
	// 	};
	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Update(backend::db::query::Update {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "my_int".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::Int(123)) })),
	// 					}
	// 				],
	// 				set: insert_set
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	assert_eq!(1, res.entries_affected);

	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Fetch(backend::db::query::Fetch {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "_id".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::String(entry_id.clone())) })),
	// 					}
	// 				]
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	let fetched_entry = res.fetched_entries.first().unwrap();
	// 	assert_eq!(
	// 		VT::String("foo bar".into()),
	// 		fetched_entry
	// 			.entry
	// 			.get("my_string")
	// 			.unwrap()
	// 			.r#type
	// 			.clone()
	// 			.unwrap()
	// 	);
	// }

	// // Delete entry
	// {
	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Delete(backend::db::query::Delete {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "_id".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::String(entry_id.clone())) })),
	// 					}
	// 				],
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	assert_eq!(1, res.entries_affected);

	// 	let res = op!([ctx] db_query_run {
	// 		database_id: Some(database_id.into()),
	// 		query: Some(backend::db::Query {
	// 			kind: Some(backend::db::query::Kind::Fetch(backend::db::query::Fetch {
	// 				collection: "test".into(),
	// 				filters: vec![
	// 					Filter {
	// 						field: "_id".into(),
	// 						kind: Some(FilterKind::Equal(Value { r#type: Some(VT::String(entry_id.clone())) })),
	// 					}
	// 				]
	// 			})),
	// 		}),
	// 	})
	// 	.await
	// 	.unwrap();
	// 	assert!(res.fetched_entries.is_empty());
	// }
}
