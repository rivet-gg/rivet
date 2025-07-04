use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;

use crate::types::LogsStreamType;

#[derive(Debug)]
pub struct Input {
	pub actor_id: util::Id,
	pub stream_type: LogsStreamType,
}

#[derive(Debug)]
pub struct Output {
	pub upload_id: Uuid,
}

#[derive(clickhouse::Row, serde::Deserialize)]
pub struct LogEntry {
	pub message: Vec<u8>,
}

#[operation]
pub async fn pegboard_actor_log_read(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let file_name = match input.stream_type {
		LogsStreamType::StdOut => "stdout.txt",
		LogsStreamType::StdErr => "stderr.txt",
	};

	let mut entries_cursor = if let util::Id::V0(actor_id) = input.actor_id {
		ctx.clickhouse()
			.await?
			.query(indoc!(
				"
				SELECT message
				FROM db_pegboard_actor_log.actor_logs
				WHERE
					actor_id = ? AND
					stream_type = ?
				ORDER BY ts ASC
	
				UNION ALL
	
				SELECT message
				FROM db_pegboard_actor_log.actor_logs2
				WHERE
					actor_id = ? AND
					stream_type = ?
				ORDER BY ts ASC
				"
			))
			.bind(actor_id)
			.bind(input.stream_type as i8)
			.bind(actor_id.to_string())
			.bind(input.stream_type as i8)
			.fetch::<LogEntry>()?
	} else {
		ctx.clickhouse()
			.await?
			.query(indoc!(
				"
				SELECT l.message
				FROM db_pegboard_runner_log.runner_logs AS l
				JOIN db_pegboard_runner.actor_runners AS ar
				ON l.runner_id = ar.runner_id
				WHERE
					ar.actor_id IN ?
					AND l.stream_type IN ?
					-- Check if the log was created during the time this actor was on this runner
					AND l.ts >= ar.started_at
					AND (ar.finished_at IS NULL OR l.ts <= ar.finished_at)
					-- Filter for actor-specific log entries using regex
					AND (l.actor_id = ar.actor_id OR l.actor_id = '')
				ORDER BY l.ts ASC
				"
			))
			.bind(input.actor_id.to_string())
			.bind(input.stream_type as i8)
			.fetch::<LogEntry>()?
	};

	let mut lines = 0;
	let mut buf = Vec::new();
	while let Some(mut entry) = entries_cursor.next().await? {
		buf.append(&mut entry.message);
		buf.push(b'\n');
		lines += 1;
	}

	tracing::info!(?lines, bytes = ?buf.len(), "read all logs");

	// Upload log
	let mime = "text/plain";
	let content_length = buf.len();
	let upload_res = op!([ctx] upload_prepare {
		bucket: "bucket-actor-log-export".into(),
		files: vec![
			backend::upload::PrepareFile {
				path: file_name.into(),
				mime: Some(mime.into()),
				content_length: content_length as u64,
				..Default::default()
			},
		],
	})
	.await?;

	let presigned_req = unwrap!(upload_res.presigned_requests.first());
	let res = reqwest::Client::new()
		.put(&presigned_req.url)
		.body(buf)
		.header(reqwest::header::CONTENT_TYPE, mime)
		.header(reqwest::header::CONTENT_LENGTH, content_length)
		.send()
		.await?;
	if res.status().is_success() {
		tracing::info!("uploaded successfully");
	} else {
		let status = res.status();
		let text = res.text().await;
		tracing::error!(?status, ?text, "failed to upload");
		bail!("failed to upload");
	}

	op!([ctx] upload_complete {
		upload_id: upload_res.upload_id,
		bucket: Some("bucket-pegboard-log-export".into()),
	})
	.await?;

	Ok(Output {
		upload_id: unwrap!(upload_res.upload_id).as_uuid(),
	})
}
