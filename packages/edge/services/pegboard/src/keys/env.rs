use std::result::Result::Ok;

use anyhow::*;
use chirp_workflow::prelude::*;
use fdb_util::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ActorKey {
	environment_id: Uuid,
	pub create_ts: i64,
	pub actor_id: Uuid,
}

impl ActorKey {
	pub fn new(environment_id: Uuid, create_ts: i64, actor_id: Uuid) -> Self {
		ActorKey {
			environment_id,
			create_ts,
			actor_id,
		}
	}

	pub fn subspace(environment_id: Uuid) -> ActorSubspaceKey {
		ActorSubspaceKey::new(environment_id)
	}
}

impl FormalKey for ActorKey {
	type Value = ActorKeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for ActorKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			ENV,
			self.environment_id,
			ACTOR,
			self.create_ts,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for ActorKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, environment_id, _, create_ts, actor_id)) =
			<(usize, Uuid, usize, i64, Uuid)>::unpack(input, tuple_depth)?;
		let v = ActorKey {
			environment_id,
			create_ts,
			actor_id,
		};

		Ok((input, v))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorKeyData {
	pub is_destroyed: bool,
	pub tags: Vec<(String, String)>,
}

pub struct ActorSubspaceKey {
	environment_id: Uuid,
}

impl ActorSubspaceKey {
	pub fn new(environment_id: Uuid) -> Self {
		ActorSubspaceKey { environment_id }
	}
}

impl TuplePack for ActorSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (ENV, self.environment_id, ACTOR);
		t.pack(w, tuple_depth)
	}
}

#[derive(Debug)]
pub struct Actor2Key {
	environment_id: Uuid,
	pub create_ts: i64,
	pub actor_id: util::Id,
}

impl Actor2Key {
	pub fn new(environment_id: Uuid, create_ts: i64, actor_id: util::Id) -> Self {
		Actor2Key {
			environment_id,
			create_ts,
			actor_id,
		}
	}

	pub fn subspace(environment_id: Uuid) -> Actor2SubspaceKey {
		Actor2SubspaceKey::new(environment_id)
	}

	pub fn subspace_with_create_ts(environment_id: Uuid, create_ts: i64) -> Actor2SubspaceKey {
		Actor2SubspaceKey::new_with_create_ts(environment_id, create_ts)
	}
}

impl FormalKey for Actor2Key {
	type Value = Actor2KeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for Actor2Key {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			ENV,
			self.environment_id,
			ACTOR2,
			self.create_ts,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for Actor2Key {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, environment_id, _, create_ts, actor_id)) =
			<(usize, Uuid, usize, i64, util::Id)>::unpack(input, tuple_depth)?;
		let v = Actor2Key {
			environment_id,
			create_ts,
			actor_id,
		};

		Ok((input, v))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actor2KeyData {
	pub is_destroyed: bool,
	pub tags: Vec<(String, String)>,
}

pub struct Actor2SubspaceKey {
	environment_id: Uuid,
	create_ts: Option<i64>,
}

impl Actor2SubspaceKey {
	pub fn new(environment_id: Uuid) -> Self {
		Actor2SubspaceKey {
			environment_id,
			create_ts: None,
		}
	}

	pub fn new_with_create_ts(environment_id: Uuid, create_ts: i64) -> Self {
		Actor2SubspaceKey {
			environment_id,
			create_ts: Some(create_ts),
		}
	}
}

impl TuplePack for Actor2SubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		let t = (ENV, self.environment_id, ACTOR2);
		offset += t.pack(w, tuple_depth)?;

		if let Some(create_ts) = &self.create_ts {
			offset += create_ts.pack(w, tuple_depth)?;
		}

		Ok(offset)
	}
}

#[derive(Debug)]
pub struct ContainerKey {
	environment_id: Uuid,
	pub create_ts: i64,
	pub actor_id: util::Id,
}

impl ContainerKey {
	pub fn new(environment_id: Uuid, create_ts: i64, actor_id: util::Id) -> Self {
		ContainerKey {
			environment_id,
			create_ts,
			actor_id,
		}
	}

	pub fn subspace(environment_id: Uuid) -> ContainerSubspaceKey {
		ContainerSubspaceKey::new(environment_id)
	}

	pub fn subspace_with_create_ts(environment_id: Uuid, create_ts: i64) -> ContainerSubspaceKey {
		ContainerSubspaceKey::new_with_create_ts(environment_id, create_ts)
	}
}

impl FormalKey for ContainerKey {
	type Value = ContainerKeyData;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		serde_json::from_slice(raw).map_err(Into::into)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		serde_json::to_vec(&value).map_err(Into::into)
	}
}

impl TuplePack for ContainerKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			ENV,
			self.environment_id,
			CONTAINER,
			self.create_ts,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for ContainerKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, environment_id, _, create_ts, actor_id)) =
			<(usize, Uuid, usize, i64, util::Id)>::unpack(input, tuple_depth)?;
		let v = ContainerKey {
			environment_id,
			create_ts,
			actor_id,
		};

		Ok((input, v))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerKeyData {
	pub is_destroyed: bool,
	pub tags: Vec<(String, String)>,
}

pub struct ContainerSubspaceKey {
	environment_id: Uuid,
	create_ts: Option<i64>,
}

impl ContainerSubspaceKey {
	pub fn new(environment_id: Uuid) -> Self {
		ContainerSubspaceKey {
			environment_id,
			create_ts: None,
		}
	}

	pub fn new_with_create_ts(environment_id: Uuid, create_ts: i64) -> Self {
		ContainerSubspaceKey {
			environment_id,
			create_ts: Some(create_ts),
		}
	}
}

impl TuplePack for ContainerSubspaceKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		let t = (ENV, self.environment_id, CONTAINER);
		offset += t.pack(w, tuple_depth)?;

		if let Some(create_ts) = &self.create_ts {
			offset += create_ts.pack(w, tuple_depth)?;
		}

		Ok(offset)
	}
}
