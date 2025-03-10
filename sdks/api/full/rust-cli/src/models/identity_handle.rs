/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// IdentityHandle : An identity handle.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct IdentityHandle {
    #[serde(rename = "account_number")]
    pub account_number: i32,
    /// The URL of this identity's avatar image.
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    /// Represent a resource's readable display name.
    #[serde(rename = "display_name")]
    pub display_name: String,
    #[serde(rename = "external")]
    pub external: Box<crate::models::IdentityExternalLinks>,
    #[serde(rename = "identity_id")]
    pub identity_id: uuid::Uuid,
    /// Whether or not this identity is registered with a linked account.
    #[serde(rename = "is_registered")]
    pub is_registered: bool,
}

impl IdentityHandle {
    /// An identity handle.
    pub fn new(account_number: i32, avatar_url: String, display_name: String, external: crate::models::IdentityExternalLinks, identity_id: uuid::Uuid, is_registered: bool) -> IdentityHandle {
        IdentityHandle {
            account_number,
            avatar_url,
            display_name,
            external: Box::new(external),
            identity_id,
            is_registered,
        }
    }
}


