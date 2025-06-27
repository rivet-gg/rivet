use foundationdb::tuple::{
	Bytes, PackResult, TupleDepth, TuplePack, TupleUnpack, VersionstampOffset,
};
use serde::{Deserialize, Serialize};

// TODO: Custom deser impl that uses arrays instead of objects?
#[derive(Clone, Serialize, Deserialize)]
pub struct Key(Vec<Vec<u8>>);

impl std::fmt::Debug for Key {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Key({})", self.len())
	}
}

impl PartialEq for Key {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}

impl Eq for Key {}

impl std::hash::Hash for Key {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for buffer in &self.0 {
			state.write(buffer);
		}
	}
}

impl Key {
	pub fn len(&self) -> usize {
		// Arbitrary 4 accounting for nesting overhead
		self.0.iter().fold(0, |acc, x| acc + x.len()) + 4 * self.0.len()
	}
}

impl TuplePack for Key {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		w.write_all(&[fdb_util::codes::NESTED])?;
		offset += 1;

		for v in self.0.iter() {
			offset += v.pack(w, tuple_depth.increment())?;
		}

		w.write_all(&[fdb_util::codes::NIL])?;
		offset += 1;

		Ok(offset)
	}
}

impl<'de> TupleUnpack<'de> for Key {
	fn unpack(mut input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		input = fdb_util::parse_code(input, fdb_util::codes::NESTED)?;

		let mut vec = Vec::new();
		while !is_end_of_tuple(input, true) {
			let (rem, v) = Bytes::unpack(input, tuple_depth.increment())?;
			input = rem;
			vec.push(v.into_owned());
		}

		input = fdb_util::parse_code(input, fdb_util::codes::NIL)?;

		Ok((input, Key(vec)))
	}
}

/// Same as Key: except when packing, it leaves off the NIL byte to allow for an open range.
#[derive(Clone, Serialize, Deserialize)]
pub struct ListKey(Vec<Vec<u8>>);

impl std::fmt::Debug for ListKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ListKey({})", self.len())
	}
}

impl TuplePack for ListKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut offset = VersionstampOffset::None { size: 0 };

		w.write_all(&[fdb_util::codes::NESTED])?;
		offset += 1;

		for v in &self.0 {
			offset += v.pack(w, tuple_depth.increment())?;
		}

		// No ending NIL byte compared to `Key::pack`

		Ok(offset)
	}
}

impl ListKey {
	pub fn len(&self) -> usize {
		// Arbitrary 4 accounting for nesting overhead
		self.0.iter().fold(0, |acc, x| acc + x.len()) + 4 * self.0.len()
	}
}

fn is_end_of_tuple(input: &[u8], nested: bool) -> bool {
	match input.first() {
		None => true,
		_ if !nested => false,
		Some(&fdb_util::codes::NIL) => Some(&fdb_util::codes::ESCAPE) != input.get(1),
		_ => false,
	}
}
