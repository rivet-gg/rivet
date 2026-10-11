//! One durable quota budget per immutable namespace, shared by all API replicas.
use anyhow::{Result, ensure};
use gas::prelude::*;
use rivet_auth_policy::errors::Signing;
use rivet_data::{generated::auth_signing_quota_v1::Data, versioned::AuthSigningQuotaData};
use vbare::OwnedVersionedData;

pub const WINDOW_MS: i64 = 60_000;
pub const BYTE_RESERVATION: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Charge {
	Application,
	Webhook,
	Bytes(u64),
}

pub fn admit(
	state: &mut Data,
	charge: Charge,
	limits: rivet_config::config::auth::SigningQuota,
	now: i64,
	dc: u16,
	generation: u64,
) -> Result<()> {
	ensure!(
		generation >= state.leader_generation
			&& (generation > state.leader_generation || dc == state.leader_datacenter),
		"stale quota authority"
	);
	if now.saturating_sub(state.window_start) >= WINDOW_MS {
		state.window_start = now;
		state.application_operations = 0;
		state.webhook_operations = 0;
		state.webhook_bytes = 0;
	}
	// Backwards time cannot start a new window or replenish its remaining budget.
	let (counter, limit, amount) = match charge {
		Charge::Application => (
			&mut state.application_operations,
			limits.application_operations,
			1,
		),
		Charge::Webhook => (&mut state.webhook_operations, limits.webhook_operations, 1),
		Charge::Bytes(bytes) => (&mut state.webhook_bytes, limits.webhook_bytes, bytes),
	};
	let next = counter
		.checked_add(amount)
		.ok_or_else(|| Signing::Throttled.build())?;
	if next > limit {
		return Err(Signing::Throttled.build());
	}
	*counter = next;
	state.leader_datacenter = dc;
	state.leader_generation = generation;
	Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
	pub namespace_id: Id,
	pub charge: Charge,
}

#[operation]
pub async fn auth_signing_quota(ctx: &OperationCtx, input: &Input) -> Result<()> {
	ensure!(
		ctx.config().is_leader(),
		"signing quota authority is leader-only"
	);
	let auth = ctx.config().auth_required()?;
	let generation = auth.jwt.issuer_generation();
	let dc = ctx.config().dc_label();
	let limits = auth.jwt.signing.quota(&input.namespace_id.to_string());
	let key = crate::storage_keys::subspace().pack(&("signing_quota", input.namespace_id));
	for _ in 0..16 {
		let read = ctx
			.op(epoxy::ops::kv::get::Input {
				key: key.clone(),
				mode: epoxy::ops::kv::get::ReadMode::Linearizable {
					target_replicas: None,
				},
			})
			.await?;
		let previous = match read.value {
			Some(value) => {
				ensure!(value.mutable, "quota state must be mutable");
				Some(value.value.ok_or_else(|| Signing::Unavailable.build())?)
			}
			None => None,
		};
		let now = rivet_util::timestamp::now();
		let mut state = if let Some(bytes) = previous.as_ref() {
			ensure!(bytes.len() <= 128, "invalid quota state");
			AuthSigningQuotaData::deserialize_with_embedded_version(bytes)?
		} else {
			Data {
				window_start: now,
				application_operations: 0,
				webhook_operations: 0,
				webhook_bytes: 0,
				leader_datacenter: dc,
				leader_generation: generation,
			}
		};
		admit(&mut state, input.charge, limits, now, dc, generation)?;
		let encoded = AuthSigningQuotaData::wrap_latest(state)
			.serialize_with_embedded_version(rivet_data::generated::AUTH_SIGNING_QUOTA_VERSION)?;
		let result = ctx
			.op(epoxy::ops::propose::Input {
				proposal: epoxy::ops::propose::Proposal {
					commands: vec![epoxy::ops::propose::Command {
						kind: epoxy::ops::propose::CommandKind::CheckAndSetCommand(
							epoxy::ops::propose::CheckAndSetCommand {
								key: key.clone(),
								expect_one_of: vec![previous],
								new_value: Some(encoded),
							},
						),
					}],
				},
				mutable: true,
				purge_cache: true,
				target_replicas: None,
			})
			.await?;
		match result {
			epoxy::ops::propose::ProposalResult::Committed => return Ok(()),
			epoxy::ops::propose::ProposalResult::ConsensusFailed {
				reason: epoxy::ops::propose::ConsensusFailedReason::ExpectedValueDoesNotMatch { .. },
			} => continue,
			// Ambiguous commits must never be retried as an uncharged admission.
			_ => return Err(Signing::Unavailable.build()),
		}
	}
	Err(Signing::Unavailable.build())
}

#[cfg(test)]
#[path = "../../tests/unit/signing_quota.rs"]
mod tests;
