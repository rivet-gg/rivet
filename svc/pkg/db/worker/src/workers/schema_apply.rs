use chirp_worker::prelude::*;
use proto::backend::{self, db::FieldPath, pkg::*};
use std::collections::HashSet;
use util_db::ais;

const MAX_FIELD_PATH_LEN: usize = 8;

#[worker(name = "db-schema-apply")]
async fn worker(ctx: OperationContext<db::msg::schema_apply::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb("db-db").await?;

	let database_id = internal_unwrap!(ctx.database_id).as_uuid();
	let schema = internal_unwrap!(ctx.schema);

	// Update schema in database & derive new schema
	let update_schema_res = rivet_pools::utils::crdb::tx(&crdb, |tx| {
		let client = ctx.chirp().clone();
		let schema = schema.clone();
		let now = ctx.ts();
		Box::pin(async move { update_schema(client, tx, now, database_id, schema).await })
	})
	.await?;
	let merged_schema = match update_schema_res {
		Ok(x) => x,
		Err(error_code) => {
			return fail(ctx.chirp(), database_id, error_code).await;
		}
	};

	msg!([ctx] db::msg::schema_apply_complete(database_id) {
		database_id: Some(database_id.into()),
	})
	.await?;

	Ok(())
}

/// Reads the schema from the database and updates it with a lock.
#[tracing::instrument(skip_all)]
async fn update_schema(
	client: chirp_client::Client,
	tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
	now: i64,
	database_id: Uuid,
	new_schema: backend::db::Schema,
) -> GlobalResult<Result<backend::db::Schema, db::msg::schema_apply_fail::ErrorCode>> {
	// Read schema
	let (old_schema_buf,) = sqlx::query_as::<_, (Vec<u8>,)>(indoc!(
		"
		SELECT schema
		FROM databases
		WHERE database_id = $1
		FOR UPDATE
		"
	))
	.bind(database_id)
	.fetch_one(&mut *tx)
	.await?;

	// Parse schema
	let old_schema = backend::db::Schema::decode(old_schema_buf.as_slice())?;

	// Merge schemas
	let merged_schema = match merge_schemas(&old_schema, &new_schema)? {
		Ok(new_schema) => new_schema,
		Err(error_code) => {
			return Ok(Err(error_code));
		}
	};

	// Encode schema
	let mut merged_schema_buf = Vec::with_capacity(merged_schema.encoded_len());
	merged_schema.encode(&mut merged_schema_buf)?;

	// Save schema
	sqlx::query(indoc!(
		"
		WITH
			_update AS (
				UPDATE databases
				SET schema = $3
				WHERE database_id = $1
				RETURNING 1
			),
			_insert AS (
				INSERT INTO database_schema_history (database_id, create_ts, schema)
				VALUES ($1, $2, $3)
				RETURNING 1

			)
		SELECT 1
		"
	))
	.bind(database_id)
	.bind(now)
	.bind(&merged_schema_buf)
	.execute(&mut *tx)
	.await?;

	Ok(Ok(merged_schema))
}

fn merge_schemas(
	old: &backend::db::Schema,
	new: &backend::db::Schema,
) -> GlobalResult<Result<backend::db::Schema, db::msg::schema_apply_fail::ErrorCode>> {
	let mut merged = old.clone();

	// Merge collections
	for new_collection in new.collections.iter() {
		if !util::check::ident_snake(&new_collection.name_id) {
			return Ok(Err(
				db::msg::schema_apply_fail::ErrorCode::InvalidCollectionName,
			));
		}

		// Get or insert existing collection
		let merged_collection = if let Some(merged_collection) = merged
			.collections
			.iter_mut()
			.find(|x| x.name_id == new_collection.name_id)
		{
			merged_collection
		} else {
			if merged.collections.len() > 128 {
				return Ok(Err(
					db::msg::schema_apply_fail::ErrorCode::TooManyCollections,
				));
			}

			// Insert new collection
			merged.collections.push(backend::db::Collection {
				name_id: new_collection.name_id.clone(),
				entry_schema: r#"{"type":"object",properties":{}}"#.into(),
				indexes: Vec::new(),
			});
			merged.collections.last_mut().unwrap()
		};

		// TODO: Validate that the entry schema is a superset
		merged_collection.entry_schema = new_collection.entry_schema.clone();

		// Merge indexes
		for new_index in new_collection.indexes.iter() {
			// Validate index
			if !util::check::ident_snake(&new_index.name_id) {
				return Ok(Err(db::msg::schema_apply_fail::ErrorCode::InvalidIndexName));
			}

			// Validate group by
			if new_index.group_by.len() > 8 {
				return Ok(Err(db::msg::schema_apply_fail::ErrorCode::TooManyGroupBy));
			}
			let mut group_by_paths = HashSet::<&Vec<String>>::new();
			for group_by in &new_index.group_by {
				// TODO: Validate field path matches valid entry schema
				if let Err(err) = validate_field_path(internal_unwrap!(group_by.field_path)) {
					return Ok(Err(err));
				}

				if group_by_paths.insert(&internal_unwrap!(group_by.field_path).field_path) {
					return Ok(Err(db::msg::schema_apply_fail::ErrorCode::DuplicateGroupBy));
				}
			}

			// Validate order by
			if new_index.order_by.len() > 1 {
				return Ok(Err(db::msg::schema_apply_fail::ErrorCode::TooManyOrderBy));
			}
			let mut order_by_paths = HashSet::<&Vec<String>>::new();
			for order_by in &new_index.order_by {
				// TODO: Validate field path matches valid entry schema
				// TODO: Validate field type & direction type
				if let Err(err) = validate_field_path(internal_unwrap!(order_by.field_path)) {
					return Ok(Err(err));
				}
				let _ = backend::db::order_by_schema::FieldType::from_i32(order_by.field_type);
				let _ = backend::db::order_by_schema::Direction::from_i32(order_by.direction);

				if order_by_paths.insert(&internal_unwrap!(order_by.field_path).field_path) {
					return Ok(Err(db::msg::schema_apply_fail::ErrorCode::DuplicateOrderBy));
				}
			}

			if let Some(merged_index) = merged_collection
				.indexes
				.iter_mut()
				.find(|x| x.name_id == new_index.name_id)
			{
				// Validate indexes are identicatl
				if !check_index_eq(merged_index, new_index) {
					return Ok(Err(db::msg::schema_apply_fail::ErrorCode::IndexChanged));
				}
			} else {
				if new_collection.indexes.len() > 8 {
					return Ok(Err(db::msg::schema_apply_fail::ErrorCode::TooManyIndexes));
				}

				// Insert new index
				merged_collection.indexes.push(new_index.clone());
			}
		}
	}

	Ok(Ok(merged))
}

fn check_index_eq(a: &backend::db::Index, b: &backend::db::Index) -> bool {
	a.group_by.len() == b.group_by.len()
		&& a.group_by.iter().zip(b.group_by.iter()).all(|(a, b)| {
			a.field_path.as_ref().map(|x| &x.field_path)
				== b.field_path.as_ref().map(|x| &x.field_path)
		}) && a.order_by.len() == b.order_by.len()
		&& a.order_by.iter().zip(b.order_by.iter()).any(|(a, b)| {
			a.field_path == b.field_path
				&& a.field_type == b.field_type
				&& a.direction == b.direction
		}) && a.include_entry == b.include_entry
}

fn validate_field_path(
	field_path: &FieldPath,
) -> Result<(), db::msg::schema_apply_fail::ErrorCode> {
	if field_path.field_path.is_empty() {
		return Err(db::msg::schema_apply_fail::ErrorCode::FieldPathEmpty);
	}
	if field_path.field_path.len() > MAX_FIELD_PATH_LEN {
		return Err(db::msg::schema_apply_fail::ErrorCode::FieldPathTooLong);
	}

	Ok(())
}

#[tracing::instrument]
async fn fail(
	client: &chirp_client::Client,
	database_id: Uuid,
	error_code: db::msg::schema_apply_fail::ErrorCode,
) -> GlobalResult<()> {
	msg!([client] db::msg::schema_apply_fail(database_id) {
		database_id: Some(database_id.into()),
		error_code: error_code as i32,
	})
	.await?;

	Ok(())
}
