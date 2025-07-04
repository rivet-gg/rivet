use std::result::Result::Ok;

use anyhow::*;
use build::types::BuildRuntime;
use chirp_workflow::prelude::*;
use fdb_util::prelude::*;
use serde::{Deserialize, Serialize};

use crate::protocol;

#[derive(Debug)]
pub struct ClientsByRemainingMemKey {
	pub flavor: protocol::ClientFlavor,
	/// MiB.
	pub remaining_mem: u64,
	pub last_ping_ts: i64,
	pub client_id: Uuid,
}

impl ClientsByRemainingMemKey {
	pub fn new(
		flavor: protocol::ClientFlavor,
		remaining_mem: u64,
		last_ping_ts: i64,
		client_id: Uuid,
	) -> Self {
		ClientsByRemainingMemKey {
			flavor,
			last_ping_ts,
			remaining_mem,
			client_id,
		}
	}

	pub fn subspace(flavor: protocol::ClientFlavor) -> ClientsByRemainingMemSubspaceKey {
		ClientsByRemainingMemSubspaceKey::new(flavor)
	}

	pub fn subspace_with_mem(
		flavor: protocol::ClientFlavor,
		remaining_mem: u64,
	) -> ClientsByRemainingMemSubspaceKey {
		ClientsByRemainingMemSubspaceKey::new_with_mem(flavor, remaining_mem)
	}

	pub fn entire_subspace() -> ClientsByRemainingMemSubspaceKey {
		ClientsByRemainingMemSubspaceKey::entire()
	}
}

impl FormalKey for ClientsByRemainingMemKey {
	/// Client workflow id.
	type Value = Uuid;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		Ok(Uuid::from_slice(raw)?)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		Ok(value.as_bytes().to_vec())
	}
}

impl TuplePack for ClientsByRemainingMemKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			DATACENTER,
			CLIENT_BY_REMAINING_MEM,
			self.flavor as usize,
			self.remaining_mem,
			self.last_ping_ts,
			self.client_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for ClientsByRemainingMemKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, flavor, remaining_mem, last_ping_ts, client_id)) =
			<(usize, usize, usize, u64, i64, Uuid)>::unpack(input, tuple_depth)?;
		let flavor = protocol::ClientFlavor::from_repr(flavor).ok_or_else(|| {
			PackError::Message(format!("invalid flavor `{flavor}` in key").into())
		})?;

		let v = ClientsByRemainingMemKey {
			flavor,
			remaining_mem,
			last_ping_ts,
			client_id,
		};

		Ok((input, v))
	}
}

pub struct ClientsByRemainingMemSubspaceKey {
	pub flavor: Option<protocol::ClientFlavor>,
	pub remaining_mem: Option<u64>,
}

impl ClientsByRemainingMemSubspaceKey {
	pub fn new(flavor: protocol::ClientFlavor) -> Self {
		ClientsByRemainingMemSubspaceKey {
			flavor: Some(flavor),
			remaining_mem: None,
		}
	}

	pub fn new_with_mem(flavor: protocol::ClientFlavor, remaining_mem: u64) -> Self {
		ClientsByRemainingMemSubspaceKey {
			flavor: Some(flavor),
			remaining_mem: Some(remaining_mem),
		}
	}

	pub fn entire() -> Self {
		ClientsByRemainingMemSubspaceKey {
			flavor: None,
			remaining_mem: None,
		}
	}
}

impl TuplePack for ClientsByRemainingMemSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		let t = (DATACENTER, CLIENT_BY_REMAINING_MEM);
		offset += t.pack(w, tuple_depth)?;

		if let Some(flavor) = &self.flavor {
			offset += (*flavor as usize).pack(w, tuple_depth)?;

			if let Some(remaining_mem) = &self.remaining_mem {
				offset += remaining_mem.pack(w, tuple_depth)?;
			}
		}

		Ok(offset)
	}
}

#[derive(Debug)]
pub struct RunnersByRemainingSlotsKey {
	pub build_id: Uuid,
	pub remaining_slots: u32,
	pub runner_id: Uuid,
}

impl RunnersByRemainingSlotsKey {
	pub fn new(build_id: Uuid, remaining_slots: u32, runner_id: Uuid) -> Self {
		RunnersByRemainingSlotsKey {
			build_id,
			remaining_slots,
			runner_id,
		}
	}

	pub fn subspace(build_id: Uuid) -> RunnersByRemainingSlotsSubspaceKey {
		RunnersByRemainingSlotsSubspaceKey::new(build_id)
	}

	pub fn subspace_with_slots(
		build_id: Uuid,
		remaining_slots: u32,
	) -> RunnersByRemainingSlotsSubspaceKey {
		RunnersByRemainingSlotsSubspaceKey::new_with_slots(build_id, remaining_slots)
	}
}

impl FormalKey for RunnersByRemainingSlotsKey {
	type Value = RunnersByRemainingSlotsKeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for RunnersByRemainingSlotsKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			DATACENTER,
			RUNNER_BY_REMAINING_SLOTS,
			self.build_id,
			self.remaining_slots,
			self.runner_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for RunnersByRemainingSlotsKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, build_id, remaining_slots, runner_id)) =
			<(usize, usize, Uuid, u32, Uuid)>::unpack(input, tuple_depth)?;

		let v = RunnersByRemainingSlotsKey {
			build_id,
			remaining_slots,
			runner_id,
		};

		Ok((input, v))
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RunnersByRemainingSlotsKeyData {
	pub client_id: Uuid,
	pub client_workflow_id: Uuid,
}

pub struct RunnersByRemainingSlotsSubspaceKey {
	pub build_id: Uuid,
	pub remaining_slots: Option<u32>,
}

impl RunnersByRemainingSlotsSubspaceKey {
	pub fn new(build_id: Uuid) -> Self {
		RunnersByRemainingSlotsSubspaceKey {
			build_id,
			remaining_slots: None,
		}
	}

	pub fn new_with_slots(build_id: Uuid, remaining_slots: u32) -> Self {
		RunnersByRemainingSlotsSubspaceKey {
			build_id,
			remaining_slots: Some(remaining_slots),
		}
	}
}

impl TuplePack for RunnersByRemainingSlotsSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		let t = (DATACENTER, RUNNER_BY_REMAINING_SLOTS, self.build_id);
		offset += t.pack(w, tuple_depth)?;

		if let Some(remaining_slots) = &self.remaining_slots {
			offset += remaining_slots.pack(w, tuple_depth)?;
		}

		Ok(offset)
	}
}

#[derive(Debug)]
pub struct PendingActorByImageIdKey {
	pub image_id: Uuid,
	pub ts: i64,
	pub actor_id: util::Id,
}

impl PendingActorByImageIdKey {
	pub fn new(image_id: Uuid, ts: i64, actor_id: util::Id) -> Self {
		PendingActorByImageIdKey {
			image_id,
			ts,
			actor_id,
		}
	}

	pub fn sister(&self) -> PendingActorKey {
		PendingActorKey {
			ts: self.ts,
			actor_id: self.actor_id,
		}
	}

	pub fn subspace(image_id: Uuid) -> PendingActorByImageIdSubspaceKey {
		PendingActorByImageIdSubspaceKey::new(image_id)
	}
}

impl FormalKey for PendingActorByImageIdKey {
	type Value = PendingActorByImageIdKeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for PendingActorByImageIdKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			DATACENTER,
			PENDING_ACTOR_BY_IMAGE_ID,
			self.image_id,
			self.ts,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for PendingActorByImageIdKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, _, image_id, ts, actor_id)) =
			<(usize, usize, usize, Uuid, i64, util::Id)>::unpack(input, tuple_depth)?;

		let v = PendingActorByImageIdKey {
			image_id,
			ts,
			actor_id,
		};

		Ok((input, v))
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PendingActorByImageIdKeyData {
	pub generation: u32,
	pub build_runtime: BuildRuntime,
	/// Millicore (1/1000 of a core).
	pub selected_cpu: u64,
	/// Bytes.
	pub selected_mem: u64,
}

pub struct PendingActorByImageIdSubspaceKey {
	pub image_id: Uuid,
}

impl PendingActorByImageIdSubspaceKey {
	pub fn new(image_id: Uuid) -> Self {
		PendingActorByImageIdSubspaceKey { image_id }
	}
}

impl TuplePack for PendingActorByImageIdSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (DATACENTER, PENDING_ACTOR_BY_IMAGE_ID, self.image_id);
		t.pack(w, tuple_depth)
	}
}

#[derive(Debug)]
pub struct PendingActorKey {
	pub ts: i64,
	pub actor_id: util::Id,
}

impl PendingActorKey {
	pub fn new(ts: i64, actor_id: util::Id) -> Self {
		PendingActorKey { ts, actor_id }
	}

	pub fn sister(&self, image_id: Uuid) -> PendingActorByImageIdKey {
		PendingActorByImageIdKey {
			image_id,
			ts: self.ts,
			actor_id: self.actor_id,
		}
	}

	pub fn subspace() -> PendingActorSubspaceKey {
		PendingActorSubspaceKey {}
	}
}

impl FormalKey for PendingActorKey {
	type Value = PendingActorKeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for PendingActorKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (DATACENTER, PENDING_ACTOR, self.ts, self.actor_id);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for PendingActorKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, _, ts, actor_id)) =
			<(usize, usize, usize, i64, util::Id)>::unpack(input, tuple_depth)?;

		let v = PendingActorKey { ts, actor_id };

		Ok((input, v))
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PendingActorKeyData {
	pub generation: u32,
	pub image_id: Uuid,
	pub build_runtime: BuildRuntime,
	/// Millicore (1/1000 of a core).
	pub selected_cpu: u64,
	/// Bytes.
	pub selected_mem: u64,
}

pub struct PendingActorSubspaceKey {}

impl TuplePack for PendingActorSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (DATACENTER, PENDING_ACTOR);
		t.pack(w, tuple_depth)
	}
}
