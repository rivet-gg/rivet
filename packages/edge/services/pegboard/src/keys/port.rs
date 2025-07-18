use std::result::Result::Ok;

use anyhow::*;
use chirp_workflow::prelude::*;
use fdb_util::prelude::*;

use crate::types::GameGuardProtocol;

#[derive(Debug)]
pub struct IngressKey {
	protocol: GameGuardProtocol,
	pub port: u16,
	pub actor_id: Uuid,
}

impl IngressKey {
	pub fn new(protocol: GameGuardProtocol, port: u16, actor_id: Uuid) -> Self {
		IngressKey {
			protocol,
			port,
			actor_id,
		}
	}
}

impl TuplePack for IngressKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			PORT,
			INGRESS,
			self.protocol as usize,
			self.port,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

#[derive(Debug)]
pub struct IngressKey2 {
	protocol: GameGuardProtocol,
	pub port: u16,
	pub actor_id: util::Id,
}

impl IngressKey2 {
	pub fn new(protocol: GameGuardProtocol, port: u16, actor_id: util::Id) -> Self {
		IngressKey2 {
			protocol,
			port,
			actor_id,
		}
	}

	pub fn subspace(protocol: GameGuardProtocol, port: u16) -> IngressSubspaceKey2 {
		IngressSubspaceKey2::new(protocol, port)
	}
}

impl FormalKey for IngressKey2 {
	type Value = ();

	fn deserialize(&self, _raw: &[u8]) -> Result<Self::Value> {
		Ok(())
	}

	fn serialize(&self, _value: Self::Value) -> Result<Vec<u8>> {
		Ok(Vec::new())
	}
}

impl TuplePack for IngressKey2 {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			PORT,
			INGRESS,
			self.protocol as usize,
			self.port,
			self.actor_id,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for IngressKey2 {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		// First parse as id, then uuid
		let (input, (_, _, protocol_variant, port, actor_id)) =
			if let Ok(res) = <(usize, usize, usize, u16, util::Id)>::unpack(input, tuple_depth) {
				res
			} else {
				let (input, (_a, _b, protocol_variant, port, actor_id)) =
					<(usize, usize, usize, u16, Uuid)>::unpack(input, tuple_depth)?;
				(
					input,
					(_a, _b, protocol_variant, port, util::Id::from(actor_id)),
				)
			};
		let protocol = GameGuardProtocol::from_repr(protocol_variant).ok_or_else(|| {
			PackError::Message(
				format!("invalid game guard protocol variant `{protocol_variant}` in key").into(),
			)
		})?;
		let v = IngressKey2 {
			protocol,
			port,
			actor_id,
		};

		Ok((input, v))
	}
}

pub struct IngressSubspaceKey2 {
	protocol: GameGuardProtocol,
	port: u16,
}

impl IngressSubspaceKey2 {
	pub fn new(protocol: GameGuardProtocol, port: u16) -> Self {
		IngressSubspaceKey2 { protocol, port }
	}
}

impl TuplePack for IngressSubspaceKey2 {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (PORT, INGRESS, self.protocol as usize, self.port);
		t.pack(w, tuple_depth)
	}
}
