
use chirp_workflow::prelude::*;
use serde_json::json;
use util::req::SendRetry;

#[derive(Debug)]
pub struct Input {
	pub user_id: Uuid,
	pub email: String
}

#[derive(Debug)]
pub struct Output {}

#[operation]
pub async fn create_contact(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	let Some(loops_config) = &ctx.config().server()?.loops else {
		return Ok(Output {});
	};
	let user_id = input.user_id;
	let loops_api_key = &loops_config.token.read();

	let created = sql_fetch_optional!(
		[ctx, (i64,)]
		"
		INSERT INTO db_loops.contacts (user_id)
		VALUES ($1)
		ON CONFLICT DO NOTHING
		RETURNING 1
		",
		user_id,
	)
	.await?
	.is_none();

	if !created {
		tracing::warn!(?user_id, "loops contact already created");
		return Ok(Output {});
	}

	let client = reqwest::Client::new();
	let body = json!({
		"email": input.email,
		"userId": format!("{}-{}", ctx.config().server()?.rivet.namespace, user_id),
	});

	client
		.post("https://app.loops.so/api/v1/contacts/create")
		.header(
			reqwest::header::AUTHORIZATION,
			format!("Bearer {}", loops_api_key),
		)
		.header(reqwest::header::CONTENT_TYPE, "application/json")
		.json(&body)
		.send_retry(5)
		.await?
		.to_global_error()
		.await?;

	Ok(Output {})
}

