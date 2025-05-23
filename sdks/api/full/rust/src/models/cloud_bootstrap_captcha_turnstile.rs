/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CloudBootstrapCaptchaTurnstile {
	#[serde(rename = "site_key")]
	pub site_key: String,
}

impl CloudBootstrapCaptchaTurnstile {
	pub fn new(site_key: String) -> CloudBootstrapCaptchaTurnstile {
		CloudBootstrapCaptchaTurnstile { site_key }
	}
}
