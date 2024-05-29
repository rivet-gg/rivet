use proto::backend::pkg::*;
use rivet_operation::prelude::*;

#[derive(clickhouse::Row, serde::Deserialize)]
struct TailEvent {
	ts: i64,
	tail_event: String,
}

#[operation(name = "cf-tail-event-read")]
async fn handle(
	ctx: OperationContext<cf::tail_event_read::Request>,
) -> GlobalResult<cf::tail_event_read::Response> {
	let clickhouse = ctx.clickhouse().await?;

	let req_query = unwrap_ref!(ctx.query);

	let order_by = if ctx.order_asc { "ASC" } else { "DESC" };

	let events = match req_query {
		cf::tail_event_read::request::Query::All(_) => {
			query_all(ctx.body(), &clickhouse, order_by).await?
		}
		cf::tail_event_read::request::Query::BeforeTs(ts) => {
			query_before_ts(ctx.body(), &clickhouse, *ts, order_by).await?
		}
		cf::tail_event_read::request::Query::AfterTs(ts) => {
			query_after_ts(ctx.body(), &clickhouse, *ts, order_by).await?
		}
		cf::tail_event_read::request::Query::TsRange(query) => {
			query_ts_range(
				ctx.body(),
				&clickhouse,
				query.after_ts,
				query.before_ts,
				order_by,
			)
			.await?
		}
	};

	Ok(cf::tail_event_read::Response { events })
}

async fn query_all(
	req: &cf::tail_event_read::Request,
	clickhouse: &ClickHousePool,
	order_by: &str,
) -> GlobalResult<Vec<cf::tail_event_read::TailEvent>> {
	let mut events_cursor = clickhouse
		.query(&formatdoc!(
			"
			SELECT ts, toJSONString(tail_event)
			FROM db_cf_log.cf_tail_events
			WHERE script_name = ?
			ORDER BY ts {order_by}
			LIMIT ?
			"
		))
		.bind(&req.script_name)
		.bind(req.count)
		.fetch::<TailEvent>()?;

	let mut events = Vec::new();
	while let Some(entry) = events_cursor.next().await? {
		events.push(convert_entry(entry));
	}

	Ok(events)
}

async fn query_before_ts(
	req: &cf::tail_event_read::Request,
	clickhouse: &ClickHousePool,
	ts: i64,
	order_by: &str,
) -> GlobalResult<Vec<cf::tail_event_read::TailEvent>> {
	let mut events_cursor = clickhouse
		.query(&formatdoc!(
			"
			SELECT ts, toJSONString(tail_event)
			FROM db_cf_log.cf_tail_events
			WHERE script_name = ? AND ts < fromUnixTimestamp64Milli(?)
			ORDER BY ts {order_by}
			LIMIT ?
			"
		))
		.bind(&req.script_name)
		.bind(ts)
		.bind(req.count)
		.fetch::<TailEvent>()?;

	let mut events = Vec::new();
	while let Some(entry) = events_cursor.next().await? {
		events.push(convert_entry(entry));
	}

	Ok(events)
}

async fn query_after_ts(
	req: &cf::tail_event_read::Request,
	clickhouse: &ClickHousePool,
	ts: i64,
	order_by: &str,
) -> GlobalResult<Vec<cf::tail_event_read::TailEvent>> {
	let mut events_cursor = clickhouse
		.query(&formatdoc!(
			"
			SELECT ts, toJSONString(tail_event)
			FROM db_cf_log.cf_tail_events
			WHERE script_name = ? AND ts > fromUnixTimestamp64Milli(?)
			ORDER BY ts {order_by}
			LIMIT ?
			"
		))
		.bind(&req.script_name)
		.bind(ts)
		.bind(req.count)
		.fetch::<TailEvent>()?;

	let mut events = Vec::new();
	while let Some(entry) = events_cursor.next().await? {
		events.push(convert_entry(entry));
	}

	Ok(events)
}

async fn query_ts_range(
	req: &cf::tail_event_read::Request,
	clickhouse: &ClickHousePool,
	after_ts: i64,
	before_ts: i64,
	order_by: &str,
) -> GlobalResult<Vec<cf::tail_event_read::TailEvent>> {
	let mut events_cursor = clickhouse
		.query(&formatdoc!(
			"
			SELECT ts, toJSONString(tail_event)
			FROM db_cf_log.cf_tail_events
			WHERE script_name = ? AND ts > fromUnixTimestamp64Milli(?) AND ts < fromUnixTimestamp64Nano(?)
			ORDER BY ts {order_by}
			LIMIT ?
			"
		))
		.bind(&req.script_name)
		.bind(after_ts)
		.bind(before_ts)
		.bind(req.count)
		.fetch::<TailEvent>()?;

	let mut events = Vec::new();
	while let Some(entry) = events_cursor.next().await? {
		events.push(convert_entry(entry));
	}

	Ok(events)
}

fn convert_entry(entry: TailEvent) -> cf::tail_event_read::TailEvent {
	cf::tail_event_read::TailEvent {
		ts: entry.ts,
		json: entry.tail_event,
	}
}
