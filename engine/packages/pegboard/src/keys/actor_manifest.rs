//! Full actor metadata manifest for one pool, stored with the existing versioned JSON metadata codec.
use anyhow::{Context, Result};
use gas::prelude::Id;
use universaldb::prelude::*;
use vbare::OwnedVersionedData;

pub struct MetadataKey {
	namespace_id: Id,
	pool_name: String,
}

impl MetadataKey {
	pub fn new(namespace_id: Id, pool_name: String) -> Self {
		MetadataKey {
			namespace_id,
			pool_name,
		}
	}
}

impl FormalChunkedKey for MetadataKey {
	type ChunkKey = MetadataChunkKey;
	type Value = rivet_data::converted::MetadataKeyData;

	fn chunk(&self, chunk: usize) -> Self::ChunkKey {
		MetadataChunkKey {
			namespace_id: self.namespace_id,
			pool_name: self.pool_name.clone(),
			chunk,
		}
	}

	fn combine(&self, chunks: Vec<Value>) -> Result<Self::Value> {
		rivet_data::versioned::MetadataKeyData::deserialize_with_embedded_version(
			&chunks
				.iter()
				.map(|x| x.value().iter().map(|x| *x))
				.flatten()
				.collect::<Vec<_>>(),
		)
		.context("failed to combine `MetadataKey`")?
		.try_into()
	}

	fn split(&self, value: Self::Value) -> Result<Vec<Vec<u8>>> {
		Ok(
			rivet_data::versioned::MetadataKeyData::wrap_latest(value.try_into()?)
				.serialize_with_embedded_version(rivet_data::PEGBOARD_RUNNER_METADATA_VERSION)?
				.chunks(universaldb::utils::CHUNK_SIZE)
				.map(|x| x.to_vec())
				.collect(),
		)
	}
}

impl TuplePack for MetadataKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			NAMESPACE,
			CONFIG,
			DATA,
			self.namespace_id,
			&self.pool_name,
			METADATA,
		);
		t.pack(w, tuple_depth)
	}
}

pub struct MetadataChunkKey {
	namespace_id: Id,
	pool_name: String,
	chunk: usize,
}

impl TuplePack for MetadataChunkKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let t = (
			NAMESPACE,
			CONFIG,
			DATA,
			self.namespace_id,
			&self.pool_name,
			METADATA,
			self.chunk,
		);
		t.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for MetadataChunkKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (_, _, _, namespace_id, pool_name, data, chunk)) =
			<(usize, usize, usize, Id, String, usize, usize)>::unpack(input, tuple_depth)?;
		if data != METADATA {
			return Err(PackError::Message("expected METADATA data".into()));
		}

		let v = MetadataChunkKey {
			namespace_id,
			pool_name,
			chunk,
		};

		Ok((input, v))
	}
}
