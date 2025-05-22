use pegboard::protocol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToManager {
	Init {
		// See `packages/edge/infra/client/manager/src/claims.rs`
		access_token: String,
	},
	ActorStateUpdate {
		actor_id: rivet_util::Id,
		generation: u32,
		state: ActorState,
	},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToRunner {
	StartActor {
		actor_id: rivet_util::Id,
		generation: u32,
		env: protocol::HashableMap<String, String>,
		metadata: protocol::Raw<protocol::ActorMetadata>,
	},
	SignalActor {
		actor_id: rivet_util::Id,
		generation: u32,
		signal: i32,
		persist_storage: bool,
	},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ActorState {
	Running,
	Exited { exit_code: Option<i32> },
}
