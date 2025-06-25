use std::result::Result::Ok;

use anyhow::*;
use chirp_workflow::prelude::*;
use fdb_util::prelude::*;

#[derive(Debug)]
pub struct ImageIdKey {
	runner_id: Uuid,
}

impl ImageIdKey {
	pub fn new(runner_id: Uuid) -> Self {
		ImageIdKey { runner_id }
	}
}

impl FormalKey for ImageIdKey {
	/// Image ID.
	type Value = Uuid;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		Ok(Uuid::from_slice(raw)?)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		Ok(value.as_bytes().to_vec())
	}
}

impl TuplePack for ImageIdKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (RUNNER, DATA, self.runner_id, IMAGE_ID);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for ImageIdKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, runner_id, _)) =
			<(usize, usize, Uuid, usize)>::unpack(input, tuple_depth)?;
		let v = ImageIdKey { runner_id };

		Ok((input, v))
	}
}

#[derive(Debug)]
pub struct RemainingSlotsKey {
	runner_id: Uuid,
}

impl RemainingSlotsKey {
	pub fn new(runner_id: Uuid) -> Self {
		RemainingSlotsKey { runner_id }
	}
}

impl FormalKey for RemainingSlotsKey {
	/// Slots.
	type Value = u32;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		Ok(u32::from_be_bytes(raw.try_into()?))
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		Ok(value.to_be_bytes().to_vec())
	}
}

impl TuplePack for RemainingSlotsKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (RUNNER, DATA, self.runner_id, REMAINING_SLOTS);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for RemainingSlotsKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, runner_id, _)) =
			<(usize, usize, Uuid, usize)>::unpack(input, tuple_depth)?;
		let v = RemainingSlotsKey { runner_id };

		Ok((input, v))
	}
}

#[derive(Debug)]
pub struct TotalSlotsKey {
	runner_id: Uuid,
}

impl TotalSlotsKey {
	pub fn new(runner_id: Uuid) -> Self {
		TotalSlotsKey { runner_id }
	}
}

impl FormalKey for TotalSlotsKey {
	/// Slots.
	type Value = u32;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		Ok(u32::from_be_bytes(raw.try_into()?))
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		Ok(value.to_be_bytes().to_vec())
	}
}

impl TuplePack for TotalSlotsKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (RUNNER, DATA, self.runner_id, TOTAL_SLOTS);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for TotalSlotsKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, runner_id, _)) =
			<(usize, usize, Uuid, usize)>::unpack(input, tuple_depth)?;
		let v = TotalSlotsKey { runner_id };

		Ok((input, v))
	}
}
