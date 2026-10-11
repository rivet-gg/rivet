//! Namespace-bound registrations for externally signed Rivet access JWTs.
//!
//! A valid signature alone never grants access: the issuer registration constrains the
//! namespace, audience, and the maximum set of signed grants that Rivet will honor.

use std::collections::HashSet;

use anyhow::{Context, Result, ensure};
use rivet_auth_policy::OwnedGrant;
use rivet_data::{
	AUTH_JWT_EXTERNAL_ISSUERS_VERSION, generated::auth_jwt_external_issuers_v1,
	versioned::AuthJwtExternalIssuersData,
};
use url::{Host, Url};
use vbare::OwnedVersionedData;

use crate::{Id, ValidatedGrantSet, decode_grants, encode_grants};

pub const MAX_ISSUERS_PER_NAMESPACE: usize = 8;

#[derive(Clone, Debug)]
pub struct ExternalIssuer {
	pub name: String,
	pub issuer: String,
	pub audience: String,
	pub jwks_url: String,
	pub algorithm: String,
	pub grant_ceiling: Vec<OwnedGrant<Id, Id>>,
}

#[derive(Clone, Debug)]
pub struct ExternalIssuerRegistry {
	pub namespace_id: Id,
	pub generation: u64,
	pub issuers: Vec<ExternalIssuer>,
}

impl ExternalIssuerRegistry {
	pub fn empty(namespace_id: Id) -> Self {
		Self {
			namespace_id,
			generation: 0,
			issuers: Vec::new(),
		}
	}

	pub fn validate(&self) -> Result<()> {
		ensure!(
			self.issuers.len() <= MAX_ISSUERS_PER_NAMESPACE,
			"too many JWT issuers"
		);
		let mut names = HashSet::new();
		let mut issuers = HashSet::new();
		for entry in &self.issuers {
			ensure!(
				!entry.name.is_empty()
					&& entry.name.len() <= 64
					&& entry
						.name
						.bytes()
						.all(|b| b.is_ascii_alphanumeric() || b == b'-'),
				"invalid JWT issuer name"
			);
			ensure!(
				names.insert(entry.name.as_str()),
				"duplicate JWT issuer name"
			);
			ensure!(
				issuers.insert(entry.issuer.as_str()),
				"duplicate JWT issuer"
			);
			ensure!(
				!entry.issuer.is_empty() && entry.issuer.len() <= 512,
				"invalid JWT issuer"
			);
			let issuer = Url::parse(&entry.issuer).context("invalid JWT issuer URL")?;
			ensure!(
				issuer.scheme() == "https"
					&& issuer.host_str().is_some()
					&& issuer.username().is_empty()
					&& issuer.password().is_none()
					&& issuer.query().is_none()
					&& issuer.fragment().is_none(),
				"JWT issuer must be an HTTPS URL without credentials, query, or fragment"
			);
			ensure!(
				!entry.audience.is_empty() && entry.audience.len() <= 256,
				"invalid JWT audience"
			);
			ensure!(
				!entry.jwks_url.is_empty() && entry.jwks_url.len() <= 2_048,
				"invalid JWKS URL"
			);
			let jwks_url = Url::parse(&entry.jwks_url).context("invalid JWKS URL")?;
			ensure!(
				jwks_url.scheme() == "https"
					&& jwks_url.port_or_known_default() == Some(443)
					&& matches!(jwks_url.host(), Some(Host::Domain(_)))
					&& jwks_url.username().is_empty()
					&& jwks_url.password().is_none()
					&& jwks_url.query().is_none()
					&& jwks_url.fragment().is_none(),
				"JWKS URL must use an HTTPS domain on port 443 without credentials, query, or fragment"
			);
			ensure!(
				matches!(entry.algorithm.as_str(), "EdDSA" | "RS256" | "ES256"),
				"unsupported JWKS signing algorithm"
			);
			ValidatedGrantSet::new(self.namespace_id, entry.grant_ceiling.clone())
				.context("invalid JWT issuer grant ceiling")?;
		}
		Ok(())
	}

	pub fn encode(&self) -> Result<Vec<u8>> {
		self.validate()?;
		let issuers = self
			.issuers
			.iter()
			.map(|entry| {
				let ceiling =
					ValidatedGrantSet::new(self.namespace_id, entry.grant_ceiling.clone())?;
				Ok(auth_jwt_external_issuers_v1::Issuer {
					name: entry.name.clone(),
					issuer: entry.issuer.clone(),
					audience: entry.audience.clone(),
					jwks_url: entry.jwks_url.clone(),
					algorithm: entry.algorithm.clone(),
					grant_ceiling: encode_grants(&ceiling)?,
				})
			})
			.collect::<Result<Vec<_>>>()?;
		Ok(
			AuthJwtExternalIssuersData::wrap_latest(auth_jwt_external_issuers_v1::Data {
				namespace_id: self.namespace_id.as_bytes(),
				generation: self.generation,
				issuers,
			})
			.serialize_with_embedded_version(AUTH_JWT_EXTERNAL_ISSUERS_VERSION)?,
		)
	}

	pub fn decode(namespace_id: Id, data: &[u8]) -> Result<Self> {
		let decoded = AuthJwtExternalIssuersData::deserialize_with_embedded_version(data)?;
		ensure!(
			Id::from_slice(&decoded.namespace_id)? == namespace_id,
			"JWT issuer registry namespace mismatch"
		);
		let issuers = decoded
			.issuers
			.into_iter()
			.map(|entry| {
				Ok(ExternalIssuer {
					name: entry.name,
					issuer: entry.issuer,
					audience: entry.audience,
					jwks_url: entry.jwks_url,
					algorithm: entry.algorithm,
					grant_ceiling: decode_grants(namespace_id, &entry.grant_ceiling)?,
				})
			})
			.collect::<Result<Vec<_>>>()?;
		let registry = Self {
			namespace_id,
			generation: decoded.generation,
			issuers,
		};
		registry.validate()?;
		Ok(registry)
	}
}

#[cfg(test)]
#[path = "../tests/unit/external_issuers.rs"]
mod tests;
