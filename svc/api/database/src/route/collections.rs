use api_helper::ctx::Ctx;
use proto::backend;
use rivet_api::models;
use rivet_operation::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::auth::Auth;

#[derive(Deserialize, Serialize)]
pub struct SingleQuery {
	key: String,
	namespace_id: Option<Uuid>,
}

fn convert_value_to_proto(value: Option<serde_json::Value>) -> GlobalResult<backend::db::Value> {
	use backend::db::value::Type as VT;

	let ty = match value {
		None | Some(serde_json::Value::Null) => VT::Null(()),
		Some(serde_json::Value::Bool(x)) => VT::Bool(x),
		Some(serde_json::Value::Number(x)) => {
			if let Some(x) = x.as_i64() {
				VT::Int(x)
			} else if let Some(x) = x.as_f64() {
				VT::Float(x)
			} else {
				internal_panic!("number was not an int or float")
			}
		}
		Some(serde_json::Value::String(x)) => VT::String(x),
		_ => internal_panic!("unsupported value type"),
	};

	Ok(backend::db::Value { r#type: Some(ty) })
}

fn convert_value_to_openapi(value: backend::db::Value) -> GlobalResult<serde_json::Value> {
	use backend::db::value::Type as VT;

	let value = match internal_unwrap_owned!(value.r#type) {
		VT::Null(_) => serde_json::Value::Null,
		VT::Bool(x) => serde_json::Value::Bool(x),
		VT::Int(x) => serde_json::Value::Number(serde_json::Number::from(x)),
		VT::Float(x) => {
			let num = internal_unwrap_owned!(serde_json::Number::from_f64(x), "invalid f64");
			serde_json::Value::Number(num)
		}
		VT::String(x) => serde_json::Value::String(x),
	};

	Ok(value)
}

fn convert_filter(filter: models::DatabaseFilter) -> GlobalResult<backend::db::Filter> {
	let kind = if let Some(value) = filter.eq {
		backend::db::filter::Kind::Equal(convert_value_to_proto(value)?)
	} else {
		internal_panic!("did not specify filter kind")
	};

	Ok(backend::db::Filter {
		field: filter.field,
		kind: Some(kind),
	})
}

fn convert_entry_proto_to_openapi(
	entry: HashMap<String, backend::db::Value>,
) -> GlobalResult<HashMap<String, serde_json::Value>> {
	let mut map = HashMap::new();
	for (key, value) in entry {
		let value = convert_value_to_openapi(value)?;
		map.insert(key, value);
	}
	Ok(map)
}

// MARK: POST /fetch
pub async fn fetch(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseFetchRequest,
) -> GlobalResult<models::DatabaseFetchResponse> {
	let database_id = ctx.auth().database(ctx.op_ctx(), body.database_id).await?;

	let filters = body
		.filters
		.iter()
		.flatten()
		.cloned()
		.map(convert_filter)
		.collect::<GlobalResult<Vec<_>>>()?;

	let res = op!([ctx] db_query_run {
		database_id: Some(database_id.into()),
		query: Some(backend::db::Query {
			kind: Some(backend::db::query::Kind::Fetch(backend::db::query::Fetch {
				collection: collection.clone(),
				filters,
			})),
		}),
	})
	.await
	.unwrap();

	let entries = res
		.fetched_entries
		.iter()
		.map(|entry| convert_entry_proto_to_openapi(entry.entry.clone()))
		.collect::<GlobalResult<Vec<_>>>()?;

	Ok(models::DatabaseFetchResponse { entries })
}

// MARK: POST /insert
pub async fn insert(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseInsertRequest,
) -> GlobalResult<models::DatabaseInsertResponse> {
	let database_id = ctx.auth().database(ctx.op_ctx(), body.database_id).await?;

	let entry = body
		.entry
		.into_iter()
		.map(|(key, value)| Ok((key, convert_value_to_proto(Some(value))?)))
		.collect::<GlobalResult<HashMap<String, _>>>()?;

	let res = op!([ctx] db_query_run {
		database_id: Some(database_id.into()),
		query: Some(backend::db::Query {
			kind: Some(backend::db::query::Kind::Insert(backend::db::query::Insert {
				collection: collection.clone(),
				entry,
			})),
		}),
	})
	.await
	.unwrap();

	Ok(models::DatabaseInsertResponse {
		ids: res.inserted_entries.clone(),
	})
}

// MARK: POST /update
pub async fn update(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseUpdateRequest,
) -> GlobalResult<models::DatabaseUpdateResponse> {
	let database_id = ctx.auth().database(ctx.op_ctx(), body.database_id).await?;

	let filters = body
		.filters
		.iter()
		.flatten()
		.cloned()
		.map(convert_filter)
		.collect::<GlobalResult<Vec<_>>>()?;

	let set = body
		.set
		.into_iter()
		.flatten()
		.map(|(key, value)| Ok((key, convert_value_to_proto(Some(value))?)))
		.collect::<GlobalResult<HashMap<String, _>>>()?;

	let res = op!([ctx] db_query_run {
		database_id: Some(database_id.into()),
		query: Some(backend::db::Query {
			kind: Some(backend::db::query::Kind::Update(backend::db::query::Update {
				collection: collection.clone(),
				filters,
				set,
			})),
		}),
	})
	.await
	.unwrap();

	Ok(models::DatabaseUpdateResponse {
		updated_count: res.entries_affected as i32,
	})
}

// MARK: POST /dlete
pub async fn delete(
	ctx: Ctx<Auth>,
	collection: String,
	body: models::DatabaseDeleteRequest,
) -> GlobalResult<models::DatabaseDeleteResponse> {
	let database_id = ctx.auth().database(ctx.op_ctx(), body.database_id).await?;

	let filters = body
		.filters
		.iter()
		.flatten()
		.cloned()
		.map(convert_filter)
		.collect::<GlobalResult<Vec<_>>>()?;

	let res = op!([ctx] db_query_run {
		database_id: Some(database_id.into()),
		query: Some(backend::db::Query {
			kind: Some(backend::db::query::Kind::Delete(backend::db::query::Delete {
				collection: collection.clone(),
				filters,
			})),
		}),
	})
	.await
	.unwrap();

	Ok(models::DatabaseDeleteResponse {
		deleted_count: res.entries_affected as i32,
	})
}
