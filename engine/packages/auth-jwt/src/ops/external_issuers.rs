//! Authoritative, versioned namespace JWKS issuer registrations.

use anyhow::{Context, Result, bail, ensure};
use epoxy::ops::propose::{
	CheckAndSetCommand, Command, CommandKind, ConsensusFailedReason, Proposal, ProposalResult,
};
use gas::prelude::*;
use universaldb::prelude::FormalKey;

use crate::{
	ExternalIssuerRegistry, Id,
	storage_keys::{self, ExternalIssuersKey},
};

#[derive(Clone, Debug)]
pub struct Snapshot {
	pub encoded: Option<Vec<u8>>,
	pub registry: ExternalIssuerRegistry,
}

pub mod get {
	use super::*;

	#[derive(Debug)]
	pub struct Input {
		pub namespace_id: Id,
	}

	/// Reads the newest reachable committed issuer registry, including its encoded CAS predecessor.
	/// This availability-first read can lag a removal or grant-ceiling reduction while a newer
	/// replica is isolated; it is not a linearizable revocation check.
	#[operation]
	pub async fn auth_jwt_external_issuers_get(
		ctx: &OperationCtx,
		input: &Input,
	) -> Result<Snapshot> {
		let key = ExternalIssuersKey {
			namespace_id: input.namespace_id,
		};
		let packed_key = storage_keys::subspace().pack(&key);
		let Some(committed) = ctx
			.op(epoxy::ops::kv::get_latest::Input { key: packed_key })
			.await?
			.value
		else {
			return Ok(Snapshot {
				encoded: None,
				registry: ExternalIssuerRegistry::empty(input.namespace_id),
			});
		};
		ensure!(
			committed.mutable,
			"JWT external issuer registry must be mutable"
		);
		let encoded = committed
			.value
			.context("JWT external issuer registry was deleted")?;
		let registry = key
			.deserialize(&encoded)
			.context("invalid JWT external issuer registry")?;
		Ok(Snapshot {
			encoded: Some(encoded),
			registry,
		})
	}
}

pub mod compare_and_set {
	use super::*;

	#[derive(Debug)]
	pub struct Input {
		pub expected: Option<Vec<u8>>,
		pub successor: ExternalIssuerRegistry,
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum Output {
		Committed,
		Conflict,
	}

	/// Only the configured leader datacenter may update an issuer registry. This is the same
	/// coordinator-owned CAS assumption as the JWT signing ring. A conflicting writer rereads and
	/// replans; never apply an obsolete snapshot after a deletion or grant-ceiling reduction.
	#[operation]
	pub async fn auth_jwt_external_issuers_compare_and_set(
		ctx: &OperationCtx,
		input: &Input,
	) -> Result<Output> {
		ensure!(
			ctx.config().is_leader(),
			"JWT issuer registration writes are leader-only"
		);
		match &input.expected {
			Some(previous) => {
				let registry =
					ExternalIssuerRegistry::decode(input.successor.namespace_id, previous)?;
				ensure!(
					registry.generation.checked_add(1) == Some(input.successor.generation),
					"JWT issuer registry generation must advance exactly once"
				);
			}
			None => ensure!(
				input.successor.generation == 1,
				"JWT issuer registry must start at generation one"
			),
		}

		let key = ExternalIssuersKey {
			namespace_id: input.successor.namespace_id,
		};
		let packed_key = storage_keys::subspace().pack(&key);
		let encoded = key.serialize(input.successor.clone())?;
		match ctx
			.op(epoxy::ops::propose::Input {
				proposal: Proposal {
					commands: vec![Command {
						kind: CommandKind::CheckAndSetCommand(CheckAndSetCommand {
							key: packed_key,
							expect_one_of: vec![input.expected.clone()],
							new_value: Some(encoded),
						}),
					}],
				},
				mutable: true,
				purge_cache: true,
				target_replicas: None,
			})
			.await?
		{
			ProposalResult::Committed => Ok(Output::Committed),
			ProposalResult::ConsensusFailed {
				reason: ConsensusFailedReason::ExpectedValueDoesNotMatch { .. },
			} => Ok(Output::Conflict),
			ProposalResult::ConsensusFailed { reason } => {
				bail!("issuer registry proposal failed: {reason:?}")
			}
		}
	}
}

pub mod mutate {
	use super::*;
	use crate::ExternalIssuer;

	#[derive(Debug)]
	pub struct Input {
		pub namespace_id: Id,
		pub name: String,
		/// `None` removes an issuer. A removal is idempotent.
		pub replacement: Option<ExternalIssuer>,
	}

	/// Replans against the newest committed generation after concurrent issuer updates. A
	/// removal never writes an older snapshot over a concurrent registration or ceiling change.
	#[operation]
	pub async fn auth_jwt_external_issuers_mutate(
		ctx: &OperationCtx,
		input: &Input,
	) -> Result<ExternalIssuerRegistry> {
		ensure!(
			ctx.config().is_leader(),
			"JWT issuer registration writes are leader-only"
		);
		ensure!(
			input
				.replacement
				.as_ref()
				.is_none_or(|entry| entry.name == input.name),
			"issuer name does not match request path"
		);
		for _ in 0..8 {
			let snapshot = ctx
				.op(get::Input {
					namespace_id: input.namespace_id,
				})
				.await?;
			let mut successor = snapshot.registry;
			let existing = successor
				.issuers
				.iter()
				.position(|entry| entry.name == input.name);
			match (&input.replacement, existing) {
				(None, None) => return Ok(successor),
				(None, Some(index)) => {
					successor.issuers.remove(index);
				}
				(Some(entry), Some(index)) => successor.issuers[index] = entry.clone(),
				(Some(entry), None) => successor.issuers.push(entry.clone()),
			}
			successor.generation = successor
				.generation
				.checked_add(1)
				.context("issuer registry generation overflow")?;
			successor.validate()?;
			match ctx
				.op(compare_and_set::Input {
					expected: snapshot.encoded,
					successor: successor.clone(),
				})
				.await?
			{
				compare_and_set::Output::Committed => return Ok(successor),
				compare_and_set::Output::Conflict => continue,
			}
		}
		bail!("JWT issuer registry changed concurrently too many times")
	}
}
