use chirp_worker::prelude::*;
use proto::backend::{self, db::FieldPath, pkg::*};
use serde_json::json;

fn field_path(path: &[&str]) -> Option<FieldPath> {
	Some(FieldPath {
		field_path: path.iter().map(|s| s.to_string()).collect(),
	})
}

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
					indexes: vec![
						backend::db::Index {
							name_id: "nothing".into(),
							group_by: vec![],
							order_by: vec![],
							include_entry: true,
						},
						backend::db::Index {
							name_id: "int".into(),
							group_by: vec![backend::db::GroupBySchema {
								field_path: field_path(&["my_int"]),
							}],
							order_by: vec![],
							include_entry: true,
						},
						backend::db::Index {
							name_id: "string_x_int".into(),
							group_by: vec![backend::db::GroupBySchema {
								field_path: field_path(&["my_string"]),
							}],
							order_by: vec![backend::db::OrderBySchema {
								field_path: field_path(&["my_int"]),
								field_type: backend::db::order_by_schema::FieldType::Int as i32,
								direction: backend::db::order_by_schema::Direction::Asc as i32,
							}],
							include_entry: true,
						},
						backend::db::Index {
							name_id: "int_x_float".into(),
							group_by: vec![backend::db::GroupBySchema {
								field_path: field_path(&["my_int"]),
							}],
							order_by: vec![backend::db::OrderBySchema {
								field_path: field_path(&["my_float"]),
								field_type: backend::db::order_by_schema::FieldType::Float as i32,
								direction: backend::db::order_by_schema::Direction::Asc as i32,
							}],
							include_entry: true,
						},
						backend::db::Index {
							name_id: "bool_x_string".into(),
							group_by: vec![backend::db::GroupBySchema {
								field_path: field_path(&["my_bool"]),
							}],
							order_by: vec![backend::db::OrderBySchema {
								field_path: field_path(&["my_string"]),
								field_type: backend::db::order_by_schema::FieldType::String as i32,
								direction: backend::db::order_by_schema::Direction::Asc as i32,
							}],
							include_entry: true,
						},
					],
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

	// Update entry by ID
	{
		let res = op!([ctx] db_query_run {
			database_id: Some(database_id.into()),
			query: Some(backend::db::Query {
				kind: Some(backend::db::query::Kind::Update(backend::db::query::Update {
					collection: "test".into(),
					entry_id: Some(entry_id),
					set: vec![
						backend::db::query::update::Set {
							field_path: field_path(&["my_int"]),
							value: "123".into(),
						}
					]
				})),
			}),
		})
		.await
		.unwrap();

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
		let entry = res.entries.first().unwrap();
		let entry_json = serde_json::from_str::<serde_json::Value>(&entry.value).unwrap();
		assert_eq!(123, entry_json.get("my_int").unwrap().as_i64().unwrap(),);
	}

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
