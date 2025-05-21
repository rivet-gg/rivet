use pegboard::protocol;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToManager {
	Init {
		runner_id: Uuid,
	},
	ActorStateUpdate {
		actor_id: Uuid,
		generation: u32,
		state: ActorState,
	},
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ToRunner {
	StartActor {
		actor_id: Uuid,
		generation: u32,
		env: protocol::HashableMap<String, String>,
		metadata: protocol::Raw<protocol::ActorMetadata>,
	},
	SignalActor {
		actor_id: Uuid,
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
