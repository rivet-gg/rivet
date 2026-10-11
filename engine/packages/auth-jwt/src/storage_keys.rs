use anyhow::Result;
use universaldb::prelude::*;

use crate::{
	ExternalIssuerRegistry, SigningKeyRing, decode_signing_key_ring, encode_signing_key_ring,
};

pub fn subspace() -> universaldb::utils::Subspace {
	universaldb::utils::Subspace::new(&(RIVET, AUTH, JWT))
}

#[derive(Debug, Clone, Copy)]
pub struct SigningKeyRingKey;

impl FormalKey for SigningKeyRingKey {
	type Value = SigningKeyRing;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		decode_signing_key_ring(raw)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		encode_signing_key_ring(&value)
	}
}

impl TuplePack for SigningKeyRingKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		KEY_RING.pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for SigningKeyRingKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, value) = usize::unpack(input, tuple_depth)?;
		if value != KEY_RING {
			return Err(PackError::Message(
				"expected JWT signing-key-ring key".into(),
			));
		}
		Ok((input, Self))
	}
}

/// One consensus-managed issuer registry per immutable namespace ID. The configured leader
/// datacenter is the sole mutation owner; other datacenters can read replicated versions.
#[derive(Debug, Clone, Copy)]
pub struct ExternalIssuersKey {
	pub namespace_id: crate::Id,
}

impl FormalKey for ExternalIssuersKey {
	type Value = ExternalIssuerRegistry;

	fn deserialize(&self, raw: &[u8]) -> Result<Self::Value> {
		ExternalIssuerRegistry::decode(self.namespace_id, raw)
	}

	fn serialize(&self, value: Self::Value) -> Result<Vec<u8>> {
		anyhow::ensure!(
			self.namespace_id == value.namespace_id,
			"issuer registry key namespace mismatch"
		);
		value.encode()
	}
}

impl TuplePack for ExternalIssuersKey {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		(EXTERNAL_ISSUERS, self.namespace_id).pack(w, tuple_depth)
	}
}

impl<'de> TupleUnpack<'de> for ExternalIssuersKey {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let (input, (tag, namespace_id)) = <(usize, crate::Id)>::unpack(input, tuple_depth)?;
		if tag != EXTERNAL_ISSUERS {
			return Err(PackError::Message(
				"expected JWT external issuers key".into(),
			));
		}
		Ok((input, Self { namespace_id }))
	}
}

#[cfg(test)]
#[path = "../tests/unit/storage_keys.rs"]
mod tests;
