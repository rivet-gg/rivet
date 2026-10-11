use acl::types::{OperationKind, ResourceKind, TargetScope};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerGrant)]
pub struct Grant {
	pub resource: ResourceKind,
	pub target: TargetScope,
	pub operations: Vec<OperationKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerUpsertRequest)]
pub struct UpsertRequest {
	/// Exact `iss` claim to trust for this namespace.
	pub issuer: String,
	/// Required, exact `aud` claim for Rivet (never optional).
	pub audience: String,
	pub jwks_url: String,
	/// Pinned signature algorithm: EdDSA, RS256, or ES256.
	pub algorithm: String,
	/// Maximum access this issuer may embed in a signed Rivet JWT.
	pub grant_ceiling: Vec<Grant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuer)]
pub struct Issuer {
	pub name: String,
	pub issuer: String,
	pub audience: String,
	pub jwks_url: String,
	pub algorithm: String,
	pub grant_ceiling: Vec<Grant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerListResponse)]
pub struct ListResponse {
	pub issuers: Vec<Issuer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerGetResponse)]
pub struct GetResponse {
	pub issuer: Issuer,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerUpsertResponse)]
pub struct UpsertResponse {
	pub issuer: Issuer,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
#[schema(as = AuthIssuerDeleteResponse)]
pub struct DeleteResponse {}
