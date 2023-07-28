use chirp_worker::prelude::*;
use proto::backend::{self, pkg::*};

#[worker(name = "db-schema-apply")]
async fn worker(ctx: OperationContext<db::msg::schema_apply::Message>) -> GlobalResult<()> {
	let crdb = ctx.crdb("db-db").await?;

	let database_id = internal_unwrap!(ctx.database_id).as_uuid();
	let schema = internal_unwrap!(ctx.schema);

	// Update schema in database & derive new schema
	let merged_schema_res = rivet_pools::utils::crdb::tx(&crdb, |tx| {
		let client = ctx.chirp().clone();
		let schema = schema.clone();
		let now = ctx.ts();
		Box::pin(async move { update_schema(client, tx, now, database_id, schema).await })
	})
	.await?;
	let merged_schema = match merged_schema_res {
		Ok(merged_schema) => merged_schema,
		Err(error_code) => {
			return fail(ctx.chirp(), database_id, error_code).await;
		}
	};

	// TODO: Generate migration script
	// TODO: Run migration
	// TODO: Don't forget update optional

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
	let merged_schema = match merge_schemas(&old_schema, &new_schema) {
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
) -> Result<backend::db::Schema, db::msg::schema_apply_fail::ErrorCode> {
	let mut merged = old.clone();

	// Merge collections
	for new_collection in new.collections.iter() {
		if !util::check::ident(&new_collection.name_id) {
			return Err(db::msg::schema_apply_fail::ErrorCode::InvalidCollectionName);
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
				return Err(db::msg::schema_apply_fail::ErrorCode::TooManyCollections);
			}

			// Insert new collection
			merged.collections.push(backend::db::Collection {
				name_id: new_collection.name_id.clone(),
				fields: Vec::new(),
			});
			merged.collections.last_mut().unwrap()
		};

		// Merge fields
		for new_field in new_collection.fields.iter() {
			// Validate field
			if !util::check::ident(&new_field.name_id) {
				return Err(db::msg::schema_apply_fail::ErrorCode::InvalidFieldName);
			}

			if let Some(merged_field) = merged_collection
				.fields
				.iter_mut()
				.find(|x| x.name_id == new_field.name_id)
			{
				// Check existing field
				if merged_field.r#type != new_field.r#type {
					return Err(db::msg::schema_apply_fail::ErrorCode::FieldTypeChanged);
				}
				if new_field.optional && !merged_field.optional {
					return Err(db::msg::schema_apply_fail::ErrorCode::FieldOptionalDisabled);
				}

				// Update field
				merged_field.optional = new_field.optional;
			} else {
				if new_collection.fields.len() > 32 {
					return Err(db::msg::schema_apply_fail::ErrorCode::TooManyFields);
				}

				// Insert new field
				merged_collection.fields.push(new_field.clone());
			}
		}
	}

	Ok(merged)
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
