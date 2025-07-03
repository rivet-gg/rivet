use foundationdb::tuple::{
	Bytes, PackResult, TupleDepth, TuplePack, TupleUnpack, VersionstampOffset,
};
use pegboard_config::runner_protocol::proto::kv;
use prost::Message;

// TODO: Custom deser impl that uses arrays instead of objects?
#[derive(Clone)]
#[repr(transparent)]
pub struct Key {
	inner: kv::Key,
}

impl Key {
	pub fn convert_vec(value: Vec<kv::Key>) -> Vec<Key> {
		// SAFETY: Key is a wrapper around kv::Kky, identical memory layout
		unsafe { std::mem::transmute(value) }
	}
}

impl std::fmt::Debug for Key {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Key({})", self.len())
	}
}

impl PartialEq for Key {
	fn eq(&self, other: &Self) -> bool {
		self.inner == other.inner
	}
}

impl Eq for Key {}

impl std::hash::Hash for Key {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for buffer in &self.inner.segments {
			state.write(buffer);
		}
	}
}

impl Key {
	pub fn len(&self) -> usize {
		self.inner.encoded_len()
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

		for v in self.inner.segments.iter() {
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

		let mut segments = Vec::new();
		while !is_end_of_tuple(input, true) {
			let (rem, v) = Bytes::unpack(input, tuple_depth.increment())?;
			input = rem;
			segments.push(v.into_owned());
		}

		input = fdb_util::parse_code(input, fdb_util::codes::NIL)?;

		Ok((
			input,
			Key {
				inner: kv::Key { segments },
			},
		))
	}
}

impl From<kv::Key> for Key {
	fn from(value: kv::Key) -> Key {
		Key { inner: value }
	}
}

impl From<Key> for kv::Key {
	fn from(value: Key) -> kv::Key {
		value.inner
	}
}

/// Same as Key: except when packing, it leaves off the NIL byte to allow for an open range.
#[derive(Clone)]
pub struct ListKey {
	inner: kv::Key,
}

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

		for v in &self.inner.segments {
			offset += v.pack(w, tuple_depth.increment())?;
		}

		// No ending NIL byte compared to `Key::pack`

		Ok(offset)
	}
}

impl ListKey {
	pub fn len(&self) -> usize {
		self.inner.encoded_len()
	}
}

impl From<kv::Key> for ListKey {
	fn from(value: kv::Key) -> ListKey {
		ListKey { inner: value }
	}
}

impl From<ListKey> for kv::Key {
	fn from(value: ListKey) -> kv::Key {
		value.inner
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
