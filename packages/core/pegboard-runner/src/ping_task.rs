// use async_trait::async_trait;
// use bytes::Bytes;
// use futures_util::{
// 	stream::{SplitSink, SplitStream},
// 	SinkExt, StreamExt,
// };
// use gas::prelude::Id;
// use gas::prelude::*;
// use http_body_util::Full;
// use hyper::upgrade::Upgraded;
// use hyper::{Response, StatusCode};
// use hyper_tungstenite::{tungstenite::Message, HyperWebsocket};
// use hyper_util::rt::TokioIo;
// use pegboard::ops::runner::update_alloc_idx::{Action, RunnerEligibility};
// use pegboard_actor_kv as kv;
// use rivet_error::*;
// use rivet_guard_core::{
// 	custom_serve::CustomServeTrait, proxy_service::ResponseBody, request_context::RequestContext,
// };
// use rivet_runner_protocol as protocol;
// use rivet_runner_protocol::*;
// use serde_json::json;
// use std::{
// 	collections::HashMap,
// 	sync::{
// 		atomic::{AtomicU32, Ordering},
// 		Arc,
// 	},
// 	time::Duration,
// };
// use tokio::sync::{Mutex, RwLock};
// use tokio_tungstenite::{
// 	tungstenite::protocol::frame::{coding::CloseCode, CloseFrame},
// 	WebSocketStream,
// };
// use universalpubsub::NextOutput;
//
// #[tracing::instrument(skip_all)]
// async fn update_ping_thread(ctx: &StandaloneCtx, conns: Arc<RwLock<Connections>>) {
// 	loop {
// 		match update_ping_thread_inner(ctx, conns.clone()).await {
// 			Ok(_) => {
// 				tracing::warn!("update ping thread thread exited early");
// 			}
// 			Err(err) => {
// 				tracing::error!(?err, "update ping thread error");
// 			}
// 		}
//
// 		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
// 	}
// }
//
// /// Updates the ping of all runners requesting a ping update at once.
// #[tracing::instrument(skip_all)]
// async fn update_ping_thread_inner(
// 	ctx: &StandaloneCtx,
// 	conns: Arc<RwLock<Connections>>,
// ) -> Result<()> {
// 	loop {
// 		tokio::time::sleep(UPDATE_PING_INTERVAL).await;
//
// 		let runners = {
// 			let mut conns = conns.write().await;
//
// 			// Select all runners that required a ping update
// 			conns
// 				.iter_mut()
// 				.map(|(runner_id, conn)| {
// 					(
// 						*runner_id,
// 						conn.workflow_id,
// 						conn.last_rtt.load(Ordering::Relaxed),
// 					)
// 				})
// 				.collect::<Vec<_>>()
// 		};
//
// 		if runners.is_empty() {
// 			continue;
// 		}
//
// 		let mut runners2 = Vec::new();
//
// 		// TODO: Parallelize
// 		// Filter out dead wfs
// 		for (runner_id, workflow_id, rtt) in runners {
// 			let Some(wf) = ctx
// 				.workflow::<pegboard::workflows::runner::Input>(workflow_id)
// 				.get()
// 				.await?
// 			else {
// 				tracing::error!(?runner_id, "workflow does not exist");
// 				continue;
// 			};
//
// 			// Only update ping if the workflow is not dead
// 			if wf.has_wake_condition {
// 				runners2.push(pegboard::ops::runner::update_alloc_idx::Runner {
// 					runner_id,
// 					action: Action::UpdatePing { rtt },
// 				});
// 			}
// 		}
//
// 		if runners2.is_empty() {
// 			continue;
// 		}
//
// 		let res = ctx
// 			.op(pegboard::ops::runner::update_alloc_idx::Input { runners: runners2 })
// 			.await?;
//
// 		for notif in res.notifications {
// 			if let RunnerEligibility::ReEligible = notif.eligibility {
// 				tracing::debug!(runner_id=?notif.runner_id, "runner has become eligible again");
//
// 				ctx.signal(pegboard::workflows::runner::CheckQueue {})
// 					.to_workflow_id(notif.workflow_id)
// 					.send()
// 					.await?;
// 			}
// 		}
// 	}
// }
