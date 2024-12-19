/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */


/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ActorEndpointType {
    #[serde(rename = "hostname")]
    Hostname,
    #[serde(rename = "path")]
    Path,

}

impl ToString for ActorEndpointType {
    fn to_string(&self) -> String {
        match self {
            Self::Hostname => String::from("hostname"),
            Self::Path => String::from("path"),
        }
    }
}

impl Default for ActorEndpointType {
    fn default() -> ActorEndpointType {
        Self::Hostname
    }
}



