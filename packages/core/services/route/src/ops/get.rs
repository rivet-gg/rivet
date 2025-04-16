use std::convert::TryInto;

use chirp_workflow::prelude::*;
use formatted_error::code::ROUTE_NOT_FOUND;
use std::collections::HashMap;

use crate::types;

#[derive(Debug)]
pub struct Input {
	pub route_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
	pub routes: Vec<types::Route>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct RouteRow {
	pub(crate) route_id: Uuid,
	namespace_id: Uuid,
	name_id: String,
	hostname: String,
	path: String,
	route_subpaths: bool,
	selector_tags: sqlx::types::Json<Box<serde_json::value::RawValue>>,
	create_ts: i64,
	update_ts: i64,
	delete_ts: Option<i64>,
}

impl TryInto<types::Route> for RouteRow {
	type Error = GlobalError;

	fn try_into(self) -> GlobalResult<types::Route> {
		Ok(types::Route {
			route_id: self.route_id,
			namespace_id: self.namespace_id,
			name_id: self.name_id,
			hostname: self.hostname,
			path: self.path,
			route_subpaths: self.route_subpaths,
			// Filter out null values on selector_tags
			selector_tags: serde_json::from_str::<HashMap<String, Option<String>>>(
				self.selector_tags.0.get(),
			)?
			.into_iter()
			.filter_map(|(k, v)| v.map(|v| (k, v)))
			.collect(),
			create_ts: self.create_ts,
			update_ts: self.update_ts,
			delete_ts: self.delete_ts,
		})
	}
}

#[operation]
pub async fn get(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	// Simple direct database query without caching
	let pool = ctx.crdb().await?;

	let rows = sql_fetch_all!(
		[ctx, RouteRow]
		"
		SELECT
			route_id,
			namespace_id,
			name_id,
			hostname,
			path,
			route_subpaths,
			selector_tags,
			create_ts,
			update_ts,
			delete_ts
		FROM db_route.routes
		WHERE route_id = ANY($1)
		AND delete_ts IS NULL
		",
		&input.route_ids
	)
	.await?;

	let routes = rows
		.into_iter()
		.map(|row| row.try_into())
		.collect::<Result<Vec<_>, _>>()?;

	Ok(Output { routes })
}
