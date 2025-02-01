use std::result::Result::Ok;

use anyhow::*;
use chirp_workflow::prelude::*;
use fdb_util::prelude::*;

#[derive(Debug)]
pub struct CreateTsKey {
	server_id: Uuid,
}

impl CreateTsKey {
	pub fn new(server_id: Uuid) -> Self {
		CreateTsKey { server_id }
	}
}

impl FormalKey for CreateTsKey {
	// Timestamp.
	type Value = i64;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		Ok(i64::from_be_bytes(raw.try_into()?))
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		Ok(value.to_be_bytes().to_vec())
	}
}

impl TuplePack for CreateTsKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = ("server", "data", self.server_id, "create_ts");
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for CreateTsKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, server_id, _)) =
			<(Cow<str>, Cow<str>, Uuid, Cow<str>)>::unpack(input, tuple_depth)?;
		let v = CreateTsKey { server_id };

		Ok((input, v))
	}
}
