use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pegboard::protocol;

#[derive(Debug, Serialize, Deserialize)]
pub enum ToManager {
	Init {
		runner_id: Uuid,
	},
	ActorStateUpdate {
		actor_id: Uuid,
		state: ActorState,
	}
}

#[derive(Serialize, Deserialize)]
pub enum ToRunner {
	StartActor {
		actor_id: Uuid,
		generation: u32,
		config: Box<protocol::ActorConfig>,
	},
	SignalActor {
		actor_id: Uuid,
		generation: u32,
		signal: i32,
		persist_storage: bool,
	},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActorState {
	Running,
	Exited {
		exit_code: Option<i32>,
	},
}
