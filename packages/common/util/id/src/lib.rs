use std::{fmt, str::FromStr};

use fdb_util::prelude::*;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum IdError {
	/// Input string length mismatch for a version.
	#[error("invalid input length: expected {expected}, got {got}")]
	InvalidLength { expected: usize, got: usize },

	#[error("invalid input length: expected at least 4 characters")]
	TooShort,

	/// Encountered a non-base36 character.
	#[error("invalid base36 character: '{0}'")]
	InvalidChar(char),

	/// Overflow or underflow in byte conversion.
	#[error("byte conversion overflow/underflow")]
	ByteError,

	/// UUID parse error.
	#[error("invalid uuid: {0}")]
	InvalidUuid(#[from] uuid::Error),

	/// Unsupported version.
	#[error("unsupported version: {0}")]
	UnsupportedVersion(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Id {
	V0(Uuid),
	V1([u8; 18]),
}

impl Id {
	/// Construct V0 from uuid.
	pub fn new_v0() -> Self {
		Id::V0(Uuid::new_v4())
	}

	pub fn new_v1(label: u16) -> Self {
		let mut data = [0u8; 18];
		data[..16].copy_from_slice(Uuid::new_v4().as_bytes());
		data[16..].copy_from_slice(&label.to_be_bytes());
		Id::V1(data)
	}

	/// Construct V0 from uuid.
	pub fn v0(uuid: Uuid) -> Self {
		Id::V0(uuid)
	}

	/// Construct V1 from components.
	pub fn v1(uuid: Uuid, label: u16) -> Self {
		let mut data = [0u8; 18];
		data[..16].copy_from_slice(uuid.as_bytes());
		data[16..].copy_from_slice(&label.to_be_bytes());
		Id::V1(data)
	}

	pub fn label(&self) -> Option<u16> {
		match self {
			Id::V1(data) => {
				let mut b = [0u8; 2];
				b.copy_from_slice(&data[16..]);
				Some(u16::from_be_bytes(b))
			}
			Id::V0(_) => None,
		}
	}
}

impl Id {
	pub fn as_v0(&self) -> Option<Uuid> {
		match self {
			Id::V0(uuid) => Some(*uuid),
			_ => None,
		}
	}

	pub fn parse(s: &str) -> Result<Self, IdError> {
		Self::from_str(s)
	}

	/// Convert the ID to its byte representation.
	pub fn as_bytes(&self) -> Vec<u8> {
		match self {
			Id::V0(uuid) => uuid.as_bytes().to_vec(),
			Id::V1(data) => {
				let mut bytes = [0; 19];
				bytes[0] = 1; // Version byte
				bytes[1..].copy_from_slice(data);

				bytes.to_vec()
			}
		}
	}

	/// Construct an ID from its byte representation.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, IdError> {
		if bytes.is_empty() {
			return Err(IdError::TooShort);
		}

		// Check if it's a UUID (16 bytes)
		if bytes.len() == 16 {
			let mut uuid_bytes = [0u8; 16];
			uuid_bytes.copy_from_slice(bytes);
			return Ok(Id::V0(Uuid::from_bytes(uuid_bytes)));
		}

		match bytes[0] {
			1 => {
				if bytes.len() != 19 {
					return Err(IdError::InvalidLength {
						expected: 19,
						got: bytes.len(),
					});
				}

				let mut data = [0u8; 18];
				data.copy_from_slice(&bytes[1..]);
				Ok(Id::V1(data))
			}
			v => Err(IdError::UnsupportedVersion(v)),
		}
	}
}

impl FromStr for Id {
	type Err = IdError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() < 4 {
			return Err(IdError::TooShort);
		}

		// V0: UUID string
		if s.len() == 36 && s.chars().nth(8) == Some('-') {
			return Ok(Id::V0(Uuid::parse_str(s)?));
		}

		let version = base36_mod256(
			base36_char_to_base10(s.chars().nth(3).expect("length should be 4"))?,
			base36_char_to_base10(s.chars().nth(2).expect("length should be 4"))?,
			base36_char_to_base10(s.chars().nth(1).expect("length should be 4"))?,
			base36_char_to_base10(s.chars().nth(0).expect("length should be 4"))?,
		);

		match version {
			1 => {
				// v1 uses 19 bytes → 30 chars base36
				let expected_len = 30;
				let got = s.len();
				if got != expected_len {
					return Err(IdError::InvalidLength {
						expected: expected_len,
						got,
					});
				}

				let buf: [u8; 19] = decode_base36(s)?;

				// slice off version byte
				let mut data = [0u8; 18];
				data.copy_from_slice(&buf[1..]);

				Ok(Id::V1(data))
			}
			v => Err(IdError::UnsupportedVersion(v)),
		}
	}
}

impl fmt::Display for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Id::V0(u) => write!(f, "{}", u.hyphenated()),
			Id::V1(data) => {
				// pack version + data into 19-byte array
				let mut temp = [0u8; 19];
				temp[0] = 1;
				temp[1..].copy_from_slice(data);
				// encode to 30-char base36
				let mut buf = [b'0'; 30];
				for i in 0..buf.len() {
					let mut rem = 0u32;
					for byte in temp.iter_mut().rev() {
						let v = (rem << 8) | (*byte as u32);
						*byte = (v / 36) as u8;
						rem = v % 36;
					}
					buf[i] = if rem < 10 {
						b'0' + (rem as u8)
					} else {
						b'a' + ((rem - 10) as u8)
					};
				}
				// safe as ASCII
				let s = unsafe { String::from_utf8_unchecked(buf.to_vec()) };
				write!(f, "{}", s)
			}
		}
	}
}

impl fmt::Debug for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

impl serde::Serialize for Id {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> serde::Deserialize<'de> for Id {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		Id::from_str(&s).map_err(serde::de::Error::custom)
	}
}

impl TuplePack for Id {
	fn pack<W: std::io::Write>(
		&self,
		w: &mut W,
		tuple_depth: TupleDepth,
	) -> std::io::Result<VersionstampOffset> {
		let mut size = 1;

		w.write_all(&[fdb_util::codes::ID])?;

		// IMPORTANT: While the normal bytes representation of a v0 ID doesn't include the version, we write
		// it here so that we can unpack without a terminating NIL.
		if let Id::V0(_) = self {
			w.write_all(&[0])?;
			size += 1;
		}

		let bytes = self.as_bytes();

		let len = u32::try_from(bytes.len())
			.map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
		size += len;

		w.write_all(&bytes)?;

		Ok(VersionstampOffset::None { size })
	}
}

impl<'de> TupleUnpack<'de> for Id {
	fn unpack(input: &[u8], tuple_depth: TupleDepth) -> PackResult<(&[u8], Self)> {
		let input = fdb_util::parse_code(input, fdb_util::codes::ID)?;
		let (input2, version) = fdb_util::parse_byte(input)?;

		let (input, slice) = if version == 0 {
			// Parse 16 bytes after version
			fdb_util::parse_bytes(input2, 16)?
		} else {
			// Parse 19 bytes including version
			fdb_util::parse_bytes(input, 19)?
		};

		let v = Id::from_bytes(slice)
			.map_err(|err| PackError::Message(format!("bad id format: {err}").into()))?;

		Ok((input, v))
	}
}

impl From<Uuid> for Id {
	fn from(uuid: Uuid) -> Self {
		Id::V0(uuid)
	}
}

impl<DB> sqlx::Type<DB> for Id
where
	DB: sqlx::Database,
	Vec<u8>: sqlx::Type<DB>,
{
	fn type_info() -> DB::TypeInfo {
		<Vec<u8> as sqlx::Type<DB>>::type_info()
	}

	fn compatible(ty: &DB::TypeInfo) -> bool {
		<Vec<u8> as sqlx::Type<DB>>::compatible(ty)
	}
}

impl<'q, DB> sqlx::Encode<'q, DB> for Id
where
	DB: sqlx::Database,
	Vec<u8>: sqlx::Encode<'q, DB>,
{
	fn encode_by_ref(
		&self,
		buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
	) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
		self.as_bytes().encode_by_ref(buf)
	}
}

impl<'r, DB> sqlx::Decode<'r, DB> for Id
where
	DB: sqlx::Database,
	Vec<u8>: sqlx::Decode<'r, DB>,
{
	fn decode(
		value: <DB as sqlx::Database>::ValueRef<'r>,
	) -> Result<Self, sqlx::error::BoxDynError> {
		let bytes = <Vec<u8> as sqlx::Decode<DB>>::decode(value)?;
		Self::from_bytes(&bytes).map_err(|e| Box::new(e) as _)
	}
}

impl sqlx::postgres::PgHasArrayType for Id {
	fn array_type_info() -> sqlx::postgres::PgTypeInfo {
		sqlx::postgres::PgTypeInfo::with_name("_bytea")
	}
}

impl Default for Id {
	fn default() -> Self {
		Id::V0(Uuid::new_v4())
	}
}

/// Decode a base36 string into a fixed-size byte array.
fn decode_base36<const N: usize>(s: &str) -> Result<[u8; N], IdError> {
	let mut data = [0u8; N];
	for c in s.chars().rev() {
		let digit = base36_char_to_base10(c)? as u32;

		let mut carry = digit;
		for i in 0..N {
			let v = (data[i] as u32) * 36 + carry;
			data[i] = (v & 0xFF) as u8;
			carry = v >> 8;
		}
		if carry != 0 {
			return Err(IdError::ByteError);
		}
	}
	Ok(data)
}

/// Converts a base36 char into a decimal number (not a byte).
fn base36_char_to_base10(c: char) -> Result<u8, IdError> {
	match c {
		'0'..='9' => Ok(c as u8 - b'0'),
		'a'..='z' => Ok(c as u8 - b'a' + 10),
		_ => return Err(IdError::InvalidChar(c)),
	}
}

/// Converts 4 base36 digits into a byte.
fn base36_mod256(a: u8, b: u8, c: u8, d: u8) -> u8 {
	let sum: u32 = (a as u32) * 46_656 + (b as u32) * 1_296 + (c as u32) * 36 + (d as u32) * 1;
	(sum % 256) as u8
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_v0_roundtrip() {
		let uuid = Uuid::new_v4();
		let s = uuid.hyphenated().to_string();
		let id = Id::from_str(&s).unwrap();
		assert_eq!(id, Id::V0(uuid));
		assert_eq!(id.to_string(), s);
	}

	#[test]
	fn test_v1_roundtrip() {
		let uuid = Uuid::new_v4();
		let label = 0xABCD;
		let id = Id::new_v1(uuid, label);
		let s = id.to_string();
		assert_eq!(s.len(), 30);
		let parsed = Id::from_str(&s).unwrap();
		assert_eq!(parsed, id);
	}
}
