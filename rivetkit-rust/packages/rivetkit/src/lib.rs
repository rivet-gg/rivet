#![recursion_limit = "256"]

pub mod action;
pub mod actor;
pub mod context;
pub mod event;
pub mod persist;
pub mod prelude;
pub mod queue;
pub mod registry;
mod serverless_listener;
pub mod sqlite;
pub mod start;
pub mod telemetry;
pub mod test;
#[cfg(test)]
mod test_fixtures;
pub mod typed_client;

pub use crate::{
	action::{Action, ActionEntry, ActionSet, Handles, Raw},
	actor::{Actor, HttpCallbackAdmission, HttpCallbackClass},
	context::{
		ConnCtx, ConnIter, Cron, CronEveryOptions, CronJobInfo, CronSetOptions, Ctx, Schedule,
		StateMut, StateRef,
	},
	event::{
		ActionCall, ConnClosed, ConnOpen, Destroy, Event, EventEntry, EventSet, HttpCall,
		HttpReply, RuntimeEvent, SerializeState, Sleep, Subscribe, WsOpen,
	},
	queue::{HandlesQueue, Queue, QueueEntry, QueueMessage, QueueSet, TypedQueueMessage},
	registry::Registry,
	sqlite::{SqliteDbExt, SqliteTransactionOptions},
	start::{Events, Hibernated, Input, Snapshot, Start, run_actor},
	typed_client::{IntoActorKey, TypedActorConnection, TypedActorHandle, TypedClientExt},
};
pub use rivetkit_client as client;
pub use rivetkit_core::actor::schedule::{
	CronFire, ScheduleErrorInfo, ScheduleKind, ScheduledEventInfo, ScheduledFireInfo,
};
pub use rivetkit_core::actor::state::OnStateChangeGuard;
pub use rivetkit_core::inspector::InspectorTabEntry;
pub use rivetkit_core::metrics::RenderedMetrics;
pub use rivetkit_core::serverless::{
	CoreServerlessRuntime, ServerlessRequest, ServerlessResponse, ServerlessStreamError,
};
pub use rivetkit_core::serverless_http;
pub use rivetkit_core::{
	ActorConfig, ActorHttpResponse, ActorKey, ActorKeySegment, ActorKv, ActorTracingConfig,
	CanHibernateWebSocket, ConnHandle, ConnId, EngineSpawnMode, HTTP_BODY_STREAM_CHANNEL_CAPACITY,
	KeepAwakeRegion, ListOpts, Request, RequestSaveOpts, Response, ResponseChunk, RuntimeMode,
	SaveStateOpts, SerializeStateReason, ServeConfig, SqliteBatchStatement, SqliteDb,
	SqliteTransaction, StateDelta, StreamingResponse, WebSocket, WsMessage,
	sqlite::{BindParam, ColumnValue, ExecResult, QueryResult},
};

pub use rivetkit_core::actor::stream::StreamSource;

// Compatibility aliases keep legacy queue types available without changing behavior.
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type CompletableQueueMessage = rivetkit_core::CompletableQueueMessage;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type EnqueueAndWaitOpts = rivetkit_core::EnqueueAndWaitOpts;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type CoreQueueMessage = rivetkit_core::QueueMessage;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type QueueNextBatchOpts = rivetkit_core::QueueNextBatchOpts;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type QueueNextOpts = rivetkit_core::QueueNextOpts;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type QueueTryNextBatchOpts = rivetkit_core::QueueTryNextBatchOpts;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type QueueTryNextOpts = rivetkit_core::QueueTryNextOpts;
#[deprecated(
	note = "Queues are deprecated. Migrate to actor streams: https://rivet.dev/actors/docs/migrate-queues-to-streams"
)]
pub type QueueWaitOpts = rivetkit_core::QueueWaitOpts;
