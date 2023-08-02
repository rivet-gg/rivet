use aes_gcm::{
	aead::{AeadCore, AeadInPlace, KeyInit, OsRng},
	Aes256Gcm, Nonce,
};
use global_error::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EntryId {
	/// The entry of the ID in the databse.
	pub entry_id: i64,

	/// The region that this entry lives in.
	///
	/// Currently not used.
	pub region_id: u8,
}

impl EntryId {
	pub fn new(entry_id: i64) -> Self {
		Self {
			entry_id,
			region_id: 0,
		}
	}

	fn cipher(key: &[u8]) -> GlobalResult<Aes256Gcm> {
		let cipher = match Aes256Gcm::new_from_slice(key) {
			Ok(x) => x,
			Err(err) => {
				tracing::error!(?err, "failed to create aes cipher");
				internal_panic!("failed to create aes cipher");
			}
		};
		Ok(cipher)
	}

	/// Converts ID from database -> user-friendly.
	///
	/// IDs are encoded in order to:
	/// 1. Include extra data in the ID (like the database region).
	/// 2. Prevent users from guessing other IDs.
	/// 3. Prevent users from guessing the number of records in the database.
	pub fn encode(&self, key: &[u8], nonce: &[u8]) -> GlobalResult<String> {
		// Serialize entry
		// TODO: Find a way to do this without allocating a vec
		let mut buf = bincode::serialize(self)?;

		// // Encrypt buffer
		// // TODO: `Aes128Gcm`
		// let cipher = Self::cipher(key)?;
		// let nonce = Nonce::from_slice(nonce);
		// match cipher.encrypt_in_place(nonce, b"", &mut buf) {
		// 	Ok(_) => {}
		// 	Err(err) => {
		// 		tracing::error!(?err, "failed to encrypt entry id");
		// 		internal_panic!("failed to encrypt entry id");
		// 	}
		// };

		// Encode base58
		let bs58 = bs58::encode(&buf).into_string();

		Ok(bs58)
	}

	/// Converts ID from user-friendly -> database.
	///
	/// See `encode_id` for more details.
	pub fn decode(key: &[u8], nonce: &[u8], encoded: &str) -> GlobalResult<Self> {
		// Decode base58
		// TODO: Do this without alloc
		let mut buf = bs58::decode(encoded).into_vec()?;

		// // Decrypt buffer
		// let cipher = Self::cipher(key)?;
		// let nonce = Nonce::from_slice(nonce);
		// match cipher.decrypt_in_place(nonce, b"", &mut buf) {
		// 	Ok(_) => {}
		// 	Err(err) => {
		// 		tracing::error!(?err, "failed to decrypt entry id");
		// 		internal_panic!("failed to decrypt entry id");
		// 	}
		// };

		// Deserialize entry
		let entry_id = bincode::deserialize(&buf)?;

		Ok(entry_id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn roundtrip() {
		let id = EntryId {
			entry_id: 1234,
			region_id: 42,
		};

		let key = Aes256Gcm::generate_key(&mut OsRng).to_vec();
		let nonce = Aes256Gcm::generate_nonce(&mut OsRng).to_vec();
		let encoded = id.encode(&key, &nonce).unwrap();
		println!("encoded: {encoded} {}", encoded.len());
		let decoded = EntryId::decode(&key, &nonce, &encoded).unwrap();

		assert_eq!(id, decoded, "ids do not match");
	}
}
