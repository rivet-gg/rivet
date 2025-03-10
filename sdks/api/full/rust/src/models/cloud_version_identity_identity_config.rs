/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// CloudVersionIdentityIdentityConfig : **Deprecated** Identity configuration for a given version.

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudVersionIdentityIdentityConfig {
	/// **Deprecated**
	#[serde(rename = "display_names", skip_serializing_if = "Option::is_none")]
	pub display_names: Option<Vec<String>>,
	/// **Deprecated**
	#[serde(rename = "avatars", skip_serializing_if = "Option::is_none")]
	pub avatars: Option<Vec<uuid::Uuid>>,
	/// **Deprecated**
	#[serde(
		rename = "custom_display_names",
		skip_serializing_if = "Option::is_none"
	)]
	pub custom_display_names: Option<Vec<crate::models::CloudVersionIdentityCustomDisplayName>>,
	/// **Deprecated**
	#[serde(rename = "custom_avatars", skip_serializing_if = "Option::is_none")]
	pub custom_avatars: Option<Vec<crate::models::CloudVersionIdentityCustomAvatar>>,
}

impl CloudVersionIdentityIdentityConfig {
	/// **Deprecated** Identity configuration for a given version.
	pub fn new() -> CloudVersionIdentityIdentityConfig {
		CloudVersionIdentityIdentityConfig {
			display_names: None,
			avatars: None,
			custom_display_names: None,
			custom_avatars: None,
		}
	}
}
