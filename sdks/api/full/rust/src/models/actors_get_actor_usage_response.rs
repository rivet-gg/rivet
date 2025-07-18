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
pub struct ActorsGetActorUsageResponse {
	#[serde(rename = "metric_names")]
	pub metric_names: Vec<String>,
	#[serde(rename = "metric_attributes")]
	pub metric_attributes: Vec<::std::collections::HashMap<String, String>>,
	#[serde(rename = "metric_types")]
	pub metric_types: Vec<String>,
	#[serde(rename = "metric_values")]
	pub metric_values: Vec<Vec<f64>>,
}

impl ActorsGetActorUsageResponse {
	pub fn new(
		metric_names: Vec<String>,
		metric_attributes: Vec<::std::collections::HashMap<String, String>>,
		metric_types: Vec<String>,
		metric_values: Vec<Vec<f64>>,
	) -> ActorsGetActorUsageResponse {
		ActorsGetActorUsageResponse {
			metric_names,
			metric_attributes,
			metric_types,
			metric_values,
		}
	}
}
