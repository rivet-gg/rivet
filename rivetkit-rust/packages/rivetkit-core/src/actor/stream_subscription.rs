//! Serializable routing declarations published in actor-type metadata.
//!
//! These describe destinations, not concrete DS consumers or processing cursors.
//! Handlers, credentials and runtime validation schemas are never exported here.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use urlpattern::{UrlPattern, UrlPatternInit, UrlPatternMatchInput};

/// A local handler's routing configuration; its actor name comes from registry ownership.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NamedStreamSubscription {
	pub stream: String,
	pub actor_key: Vec<String>,
}

impl NamedStreamSubscription {
	fn pattern(&self) -> Result<UrlPattern> {
		ensure!(
			self.stream.starts_with('/') && self.stream.len() <= 2048,
			"Stream pattern must be an absolute pathname of at most 2048 bytes"
		);
		Ok(UrlPattern::parse(
			UrlPatternInit {
				pathname: Some(self.stream.clone()),
				..Default::default()
			},
			Default::default(),
		)?)
	}

	pub fn validate(&self) -> Result<()> {
		let pattern = self.pattern()?;
		for part in &self.actor_key {
			if let Some(capture) = part.strip_prefix(':') {
				ensure!(
					pattern
						.pathname
						.group_name_list
						.iter()
						.any(|name| name == capture),
					"Unknown stream capture: {capture}"
				);
			}
		}
		Ok(())
	}

	/// Match without interpolating captures back into URL syntax. Percent-encoded
	/// path components stay encoded, matching URLPattern's pathname behavior.
	pub fn resolve_key(&self, path: &str) -> Result<Option<Vec<String>>> {
		let result = self
			.pattern()?
			.exec(UrlPatternMatchInput::Init(UrlPatternInit {
				pathname: Some(format!("/{}", path.trim_start_matches('/'))),
				..Default::default()
			}))?;
		let Some(result) = result else {
			return Ok(None);
		};
		self.actor_key
			.iter()
			.map(|part| {
				if let Some(capture) = part.strip_prefix(':') {
					result
						.pathname
						.groups
						.get(capture)
						.and_then(|value| value.clone())
						.ok_or_else(|| anyhow::anyhow!("Missing stream capture: {capture}"))
				} else {
					Ok(part.clone())
				}
			})
			.collect::<Result<Vec<_>>>()
			.map(Some)
	}
}

impl super::config::InputStreamConfig {
	pub fn validate(&self) -> Result<()> {
		for (name, subscription) in &self.subscriptions {
			ensure!(
				!name.is_empty()
					&& name != "primary"
					&& name.len() <= 128
					&& name
						.bytes()
						.all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
				"Invalid or reserved stream subscription name: {name}"
			);
			subscription.validate()?;
		}
		for (name, config) in &self.dead_letters {
			ensure!(
				(name == "primary" && self.primary) || self.subscriptions.contains_key(name),
				"Dead-letter handler is not configured"
			);
			if let Some(path) = &config.stream {
				ensure!(
					!path.is_empty()
						&& !path.starts_with('/')
						&& !path.starts_with("__ds/")
						&& !path.contains(['?', '#', '\\']),
					"Invalid dead-letter stream path"
				);
			}
		}

		Ok(())
	}

	pub fn declarations(&self, actor_name: &str) -> BTreeMap<String, StreamSubscription> {
		let mut declarations: BTreeMap<_, _> = self
			.subscriptions
			.iter()
			.map(|(name, sub)| {
				(
					name.clone(),
					StreamSubscription {
						stream: sub.stream.clone(),
						destination: StreamDestination::ActorKey(ActorKeyDestination {
							actor_name: actor_name.to_owned(),
							key: sub.actor_key.clone(),
						}),
					},
				)
			})
			.collect();
		if self.primary {
			declarations.insert("primary".into(), StreamSubscription::primary(actor_name));
		}
		declarations
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamSubscription {
	pub stream: String,
	pub destination: StreamDestination,
}

/// Externally tagged so metadata contains exactly one destination variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum StreamDestination {
	ActorId(ActorIdDestination),
	ActorKey(ActorKeyDestination),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorIdDestination {
	pub actor_name: String,
	pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorKeyDestination {
	pub actor_name: String,
	pub key: Vec<String>,
}

impl StreamSubscription {
	pub(crate) fn primary(actor_name: &str) -> Self {
		// URLPattern treats these characters as pattern syntax. The registered
		// actor name is a literal path segment, even if it contains punctuation.
		let mut url = url::Url::parse("https://actor.invalid/").expect("static URL");
		url.path_segments_mut()
			.expect("static URL path")
			.clear()
			.push(actor_name);
		let mut literal = String::new();
		for ch in url.path().trim_start_matches('/').chars() {
			if matches!(ch, '+' | '*' | '?' | ':' | '{' | '}' | '(' | ')' | '\\') {
				literal.push('\\');
			}
			literal.push(ch);
		}
		Self {
			stream: format!("/actors/{literal}/:actorId"),
			destination: StreamDestination::ActorId(ActorIdDestination {
				actor_name: actor_name.to_owned(),
				id: ":actorId".to_owned(),
			}),
		}
	}
}

#[cfg(test)]
#[path = "../../tests/unit/stream_subscription.rs"]
mod tests;
