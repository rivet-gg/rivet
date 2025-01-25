use chirp_workflow::prelude::*;
use sqlite_util::SqlitePoolExt;

pub async fn run(ctx: &mut WorkflowCtx) -> GlobalResult<()> {
	ctx.activity(MigrateInitInput {}).await?;

	Ok(())
}

#[derive(Debug, Serialize, Deserialize, Hash)]
struct MigrateInitInput {}

#[activity(MigrateInit)]
async fn migrate_init(ctx: &ActivityCtx, _input: &MigrateInitInput) -> GlobalResult<()> {
	let mut tx = ctx.sqlite().await?.begin_immediate().await?;

	sql_execute!(
		[ctx, @tx &mut tx]
		"
		CREATE TABLE state (
			env_id BLOB NOT NULL, -- UUID
			tags BLOB NOT NULL, -- JSONB, map<string, string>
			resources_cpu_millicores INT NOT NULL,
			resources_memory_mib INT NOT NULL,

			lifecycle_kill_timeout_ms INT NOT NULL,
			lifecycle_durable BOOLEAN NOT NULL DEFAULT false,
			
			create_ts INT NOT NULL,
			start_ts INT,
			connectable_ts INT,
			finish_ts INT,
			destroy_ts INT,

			image_id BLOB NOT NULL, -- UUID
			args BLOB NOT NULL, -- JSONB, list<string>
			network_mode INT NOT NULL, -- ds::types::NetworkMode
			environment BLOB NOT NULL -- JSONB, map<string, string>
		) STRICT;

		CREATE TABLE pegboard (
			client_id BLOB, -- UUID,
			client_wan_hostname TEXT,
		) STRICT;

		CREATE TABLE server_ports_gg (
			port_name TEXT PRIMARY KEY,
			port_number INT,
			gg_port INT NOT NULL,
			protocol INT NOT NULL -- ds::types::GameGuardProtocol
		) STRICT;

		CREATE TABLE server_ports_host (
			port_name TEXT PRIMARY KEY,
			port_number INT,
			protocol INT NOT NULL -- ds::types::HostProtocol
		) STRICT;

		CREATE TABLE server_proxied_ports (
			label TEXT PRIMARY KEY,
			ip TEXT NOT NULL,
			source INT NOT NULL
		) STRICT;
		",
	)
	.await?;

	tx.commit().await?;

	Ok(())
}
