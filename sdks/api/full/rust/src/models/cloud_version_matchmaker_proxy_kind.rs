/*
 * Rivet API
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.0.1
 *
 * Generated by: https://openapi-generator.tech
 */

/// CloudVersionMatchmakerProxyKind : Range of ports that can be connected to. `game_guard` (default) proxies all traffic through [Game Guard](https://rivet.gg/docs/dynamic-servers/concepts/game-guard) to mitigate DDoS attacks and provide TLS termination. `none` sends traffic directly to the game server. If configured, `network_mode` must equal `host`. Read more about host networking [here](https://rivet.gg/docs/dynamic-servers/concepts/host-bridge-networking). Only available on Rivet Open Source & Enterprise.  ### Related - /docs/dynamic-servers/concepts/game-guard - cloud.version.matchmaker.PortProtocol

/// Range of ports that can be connected to. `game_guard` (default) proxies all traffic through [Game Guard](https://rivet.gg/docs/dynamic-servers/concepts/game-guard) to mitigate DDoS attacks and provide TLS termination. `none` sends traffic directly to the game server. If configured, `network_mode` must equal `host`. Read more about host networking [here](https://rivet.gg/docs/dynamic-servers/concepts/host-bridge-networking). Only available on Rivet Open Source & Enterprise.  ### Related - /docs/dynamic-servers/concepts/game-guard - cloud.version.matchmaker.PortProtocol
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum CloudVersionMatchmakerProxyKind {
	#[serde(rename = "none")]
	None,
	#[serde(rename = "game_guard")]
	GameGuard,
}

impl ToString for CloudVersionMatchmakerProxyKind {
	fn to_string(&self) -> String {
		match self {
			Self::None => String::from("none"),
			Self::GameGuard => String::from("game_guard"),
		}
	}
}

impl Default for CloudVersionMatchmakerProxyKind {
	fn default() -> CloudVersionMatchmakerProxyKind {
		Self::None
	}
}
