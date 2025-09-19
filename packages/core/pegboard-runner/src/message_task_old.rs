use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{
	stream::{SplitSink, SplitStream},
	SinkExt, StreamExt,
};
use gas::prelude::Id;
use gas::prelude::*;
use http_body_util::Full;
use hyper::upgrade::Upgraded;
use hyper::{Response, StatusCode};
use hyper_tungstenite::{tungstenite::Message, HyperWebsocket};
use hyper_util::rt::TokioIo;
use pegboard::ops::runner::update_alloc_idx::{Action, RunnerEligibility};
use pegboard_actor_kv as kv;
use rivet_error::*;
use rivet_guard_core::{
	custom_serve::CustomServeTrait, proxy_service::ResponseBody, request_context::RequestContext,
};
use rivet_runner_protocol as protocol;
use rivet_runner_protocol::*;
use serde_json::json;
use std::{
	collections::HashMap,
	sync::{
		atomic::{AtomicU32, Ordering},
		Arc,
	},
	time::Duration,
};
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{
	tungstenite::protocol::frame::{coding::CloseCode, CloseFrame},
	WebSocketStream,
};
use universalpubsub::NextOutput;

#[tracing::instrument(skip_all)]
async fn msg_thread(ctx: &StandaloneCtx, conns: Arc<RwLock<Connections>>) {
	loop {
		match msg_thread_inner(ctx, conns.clone()).await {
			Ok(_) => {
				tracing::warn!("msg thread exited early");
			}
			Err(err) => {
				tracing::error!(?err, "msg thread error");
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
	}
}

#[tracing::instrument(skip_all)]
async fn msg_thread_inner(ctx: &StandaloneCtx, conns: Arc<RwLock<Connections>>) -> Result<()> {
	// Listen for commands from runner workflows
	let runner_id: Id = todo!();
	let topic = pegboard::pubsub_subjects::RunnerReceiverSubject::new(runner_id).to_string();
	let mut sub = ctx.ups()?.subscribe(&topic).await?;

	loop {
		tokio::select! {
			msg = sub.next() => {
				// Parse message
				let msg = match msg? {
					NextOutput::Message(ups_msg) => {
						tracing::info!(
							payload_len = ups_msg.payload.len(),
							"received message from pubsub, forwarding to WebSocket"
						);

						// Parse message
						let msg = match versioned::ToClient::deserialize_with_embedded_version(
							&ups_msg.payload,
						) {
							Result::Ok(x) => x,
							Err(err) => {
								tracing::error!(?err, "failed to parse tunnel message");
								continue;
							}
						};

						msg
					}
					NextOutput::Unsubscribed => {
						tracing::info!("runner subscription unsubscribed");
						// TODO: Handle close like below
						return Ok(());
					}
				};

				{
					let conns = conns.read().await;

					// Send command to socket
					if let Some(conn) = conns.get(&runner_id) {
						let buf = versioned::ToClient::serialize(
							versioned::ToClient::latest(msg),
							conn.protocol_version
						)?;
						conn.tx.lock().await.send(Message::Binary(buf.into())).await?;
					} else {
						tracing::debug!(
							?runner_id,
							"received command for runner that isn't connected, ignoring"
						);
					}
				}
			}
			// msg = close_sub.next() => {
			// 	let msg = msg?;
			//
			// 	{
			// 		let conns = conns.read().await;
			//
			// 		// Close socket
			// 		if let Some(conn) = conns.get(&msg.runner_id) {
			// 			tracing::info!(runner_id = ?msg.runner_id, "received close ws event, closing socket");
			//
			// 			let close_frame = err_to_close_frame(WsError::Eviction.build());
			// 			conn.tx.lock().await.send(Message::Close(Some(close_frame))).await?;
			// 		} else {
			// 			tracing::debug!(
			// 				runner_id=?msg.runner_id,
			// 				"received close command for runner that isn't connected, ignoring"
			// 			);
			// 		}
			// 	}
			// }
		}
	}
}

async fn handle_messages(
	ctx: &StandaloneCtx,
	rx: &mut futures_util::stream::SplitStream<HyperWebSocketStream>,
	runner_id: Id,
	conn: &Connection,
) -> Result<()> {
	// Receive messages from socket
	while let Some(msg) = rx.next().await {
		let buf = match msg? {
			Message::Binary(buf) => buf,
			Message::Ping(_) => continue,
			Message::Close(_) => bail!("socket closed {}", runner_id),
			msg => {
				tracing::warn!(?runner_id, ?msg, "unexpected message");
				continue;
			}
		};

		let packet = versioned::ToServer::deserialize(&buf, conn.protocol_version)?;

		match packet {
			ToServer::ToServerPing(ping) => {
				let rtt = util::timestamp::now().saturating_sub(ping.ts).try_into()?;

				conn.last_rtt.store(rtt, Ordering::Relaxed);
			}
			// Process KV request
			ToServer::ToServerKvRequest(req) => {
				let actor_id = match Id::parse(&req.actor_id) {
					Ok(actor_id) => actor_id,
					Err(err) => {
						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: KvResponseData::KvErrorResponse(KvErrorResponse {
									message: err.to_string(),
								}),
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;

						continue;
					}
				};

				let actors_res = ctx
					.op(pegboard::ops::actor::get_runner::Input {
						actor_ids: vec![actor_id],
					})
					.await?;
				let actor_belongs = actors_res
					.actors
					.first()
					.map(|x| x.runner_id == runner_id)
					.unwrap_or_default();

				// Verify actor belongs to this runner
				if !actor_belongs {
					let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
						ToClientKvResponse {
							request_id: req.request_id,
							data: KvResponseData::KvErrorResponse(KvErrorResponse {
								message: "given actor does not belong to runner".to_string(),
							}),
						},
					));

					let buf = packet.serialize(conn.protocol_version)?;
					conn.tx
						.lock()
						.await
						.send(Message::Binary(buf.into()))
						.await?;

					continue;
				}

				// TODO: Add queue and bg thread for processing kv ops
				// Run kv operation
				match req.data {
					KvRequestData::KvGetRequest(body) => {
						let res = kv::get(&*ctx.udb()?, actor_id, body.keys).await;

						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: match res {
									Ok((keys, values, metadata)) => {
										KvResponseData::KvGetResponse(KvGetResponse {
											keys,
											values,
											metadata,
										})
									}
									Err(err) => KvResponseData::KvErrorResponse(KvErrorResponse {
										// TODO: Don't return actual error?
										message: err.to_string(),
									}),
								},
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;
					}
					KvRequestData::KvListRequest(body) => {
						let res = kv::list(
							&*ctx.udb()?,
							actor_id,
							body.query,
							body.reverse.unwrap_or_default(),
							body.limit.map(TryInto::try_into).transpose()?,
						)
						.await;

						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: match res {
									Ok((keys, values, metadata)) => {
										KvResponseData::KvListResponse(KvListResponse {
											keys,
											values,
											metadata,
										})
									}
									Err(err) => KvResponseData::KvErrorResponse(KvErrorResponse {
										// TODO: Don't return actual error?
										message: err.to_string(),
									}),
								},
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;
					}
					KvRequestData::KvPutRequest(body) => {
						let res = kv::put(&*ctx.udb()?, actor_id, body.keys, body.values).await;

						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: match res {
									Ok(()) => KvResponseData::KvPutResponse,
									Err(err) => KvResponseData::KvErrorResponse(KvErrorResponse {
										// TODO: Don't return actual error?
										message: err.to_string(),
									}),
								},
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;
					}
					KvRequestData::KvDeleteRequest(body) => {
						let res = kv::delete(&*ctx.udb()?, actor_id, body.keys).await;

						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: match res {
									Ok(()) => KvResponseData::KvDeleteResponse,
									Err(err) => KvResponseData::KvErrorResponse(KvErrorResponse {
										// TODO: Don't return actual error?
										message: err.to_string(),
									}),
								},
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;
					}
					KvRequestData::KvDropRequest => {
						let res = kv::delete_all(&*ctx.udb()?, actor_id).await;

						let packet = versioned::ToClient::latest(ToClient::ToClientKvResponse(
							ToClientKvResponse {
								request_id: req.request_id,
								data: match res {
									Ok(()) => KvResponseData::KvDropResponse,
									Err(err) => KvResponseData::KvErrorResponse(KvErrorResponse {
										// TODO: Don't return actual error?
										message: err.to_string(),
									}),
								},
							},
						));

						let buf = packet.serialize(conn.protocol_version)?;
						conn.tx
							.lock()
							.await
							.send(Message::Binary(buf.into()))
							.await?;
					}
				}
			}
			protocol::ToServer::ToServerTunnelMessage(tunnel_msg) => {
				todo!()
			}
			// Forward to runner wf
			protocol::ToServer::ToServerInit(_)
			| protocol::ToServer::ToServerEvents(_)
			| protocol::ToServer::ToServerAckCommands(_)
			| protocol::ToServer::ToServerStopping => {
				ctx.signal(pegboard::workflows::runner::Forward {
					inner: protocol::ToServer::try_from(packet)?,
				})
				.to_workflow_id(conn.workflow_id)
				.send()
				.await?;
			}
		}
	}

	bail!("stream closed {runner_id}");
}
