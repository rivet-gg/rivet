use std::convert::TryInto;

use chirp_workflow::prelude::*;

#[derive(sqlx::FromRow)]
struct ClientRow {
	client_id: Uuid,
	total_cpu: i64,
	total_memory: i64,
}

#[derive(Debug)]
pub struct Input {
	pub client_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
	pub clients: Vec<Client>,
}

#[derive(Debug)]
pub struct Client {
	pub client_id: Uuid,
	pub usage: Stats,
}

#[derive(Debug)]
pub struct Stats {
	/// Mhz
	pub cpu: u32,
	/// MiB
	pub memory: u32,
	/// MiB
	pub disk: u32,
}

#[operation]
pub async fn pegboard_client_usage_get(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	// TODO: Pull from prometheus

	let clients = input
		.client_ids
		.iter()
		.map(|client_id| {
			Ok(Client {
				client_id: *client_id,
				usage: Stats {
					cpu: 0,
					memory: 0,
					disk: 0, // TODO:
				},
			})
		})
		.collect::<GlobalResult<_>>()?;

	Ok(Output { clients })
}
