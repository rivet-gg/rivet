/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// CloudMatchmakerDevelopmentPort : A port configuration used to create development tokens.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudMatchmakerDevelopmentPort {
	#[serde(rename = "port", skip_serializing_if = "Option::is_none")]
	pub port: Option<i32>,
	#[serde(rename = "port_range", skip_serializing_if = "Option::is_none")]
	pub port_range: Option<Box<crate::models::CloudVersionMatchmakerPortRange>>,
	#[serde(rename = "protocol")]
	pub protocol: crate::models::CloudVersionMatchmakerPortProtocol,
}

impl CloudMatchmakerDevelopmentPort {
	/// A port configuration used to create development tokens.
	pub fn new(
		protocol: crate::models::CloudVersionMatchmakerPortProtocol,
	) -> CloudMatchmakerDevelopmentPort {
		CloudMatchmakerDevelopmentPort {
			port: None,
			port_range: None,
			protocol,
		}
	}
}
