use std::time::Duration;

use anyhow::*;
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const CURRENT_KID: &str = "v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	iat: i64,
	exp: i64,
	jti: Uuid,
	entitlements: Vec<Entitlement>,
}

impl Claims {
	pub fn new(entitlement: Entitlement, ttl: Duration) -> Self {
		let iat = crate::utils::now();
		let exp = iat + ttl.as_millis() as i64;

		Claims {
			iat,
			exp,
			jti: Uuid::new_v4(),
			entitlements: vec![entitlement],
		}
	}

	pub fn encode(&self, secret: &[u8]) -> Result<String> {
		let mut header = jwt::Header::new(jwt::Algorithm::HS256);
		header.kid = Some(CURRENT_KID.to_string());

		let token = jwt::encode(&header, &self, &jwt::EncodingKey::from_secret(secret))?;

		Ok(token)
	}

	pub fn decode(token: &str, secret: &[u8]) -> Result<Self> {
		let header = jwt::decode_header(token)?;
		let kid = header.kid.context("token missing kid")?;

		ensure!(kid == CURRENT_KID, "invalid kid");

		let token_data = jwt::decode::<Self>(
			token,
			&jwt::DecodingKey::from_secret(secret),
			&jwt::Validation::default(),
		)?;

		Ok(token_data.claims)
	}

	pub fn ent(&self) -> &[Entitlement] {
		&self.entitlements
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entitlement {
	Runner { runner_id: Uuid },
}
