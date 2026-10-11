use super::*;
use serde_json::json;

#[test]
fn destinations_use_the_agreed_externally_tagged_objects() {
	let primary = StreamSubscription::primary("customer");
	assert_eq!(
		serde_json::to_value(&primary).unwrap(),
		json!({
			"stream": "/actors/customer/:actorId",
			"destination": {"actorId": {"actorName": "customer", "id": ":actorId"}}
		})
	);
	let custom = json!({
		"stream": "/tenants/:tenant/orders/:customer",
		"destination": {"actorKey": {"actorName": "customer", "key": [":tenant", ":customer"]}}
	});
	let parsed: StreamSubscription = serde_json::from_value(custom.clone()).unwrap();
	assert_eq!(serde_json::to_value(parsed).unwrap(), custom);
}

#[test]
fn destinations_reject_ambiguous_or_underspecified_shapes() {
	for destination in [
		json!({"actorId": "123"}),
		json!({"actorId": {"id": "123"}}),
		json!({"actorKey": {"actorName": "customer", "key": "123"}}),
		json!({"actorId": {"actorName": "customer", "id": "123"}, "actorKey": {"actorName": "customer", "key": ["123"]}}),
		json!({"actorId": {"actorName": "customer", "id": "123", "token": "must-not-be-metadata"}}),
	] {
		assert!(serde_json::from_value::<StreamDestination>(destination).is_err());
	}
}

#[test]
fn primary_actor_names_are_literal_in_the_pattern() {
	assert_eq!(
		StreamSubscription::primary("customer(v2)").stream,
		"/actors/customer\\(v2\\)/:actorId"
	);
}

#[test]
fn named_patterns_validate_captures_and_match_whole_paths() {
	let sub = NamedStreamSubscription {
		stream: "/tenants/:tenant/orders/:customer".into(),
		actor_key: vec![":tenant".into(), "literal".into(), ":customer".into()],
	};
	sub.validate().unwrap();
	assert_eq!(
		sub.resolve_key("tenants/acme/orders/alice").unwrap(),
		Some(vec!["acme".into(), "literal".into(), "alice".into()])
	);
	assert_eq!(
		sub.resolve_key("tenants/acme/orders/alice/extra").unwrap(),
		None
	);
	assert_eq!(
		sub.resolve_key("tenants/a%2Fb/orders/alice")
			.unwrap()
			.unwrap()[0],
		"a%2Fb"
	);
	let invalid = NamedStreamSubscription {
		actor_key: vec![":missing".into()],
		..sub
	};
	assert!(invalid.validate().is_err());
}

#[test]
fn named_config_reserves_primary_and_exports_without_primary_handler() {
	let mut config = super::super::config::InputStreamConfig {
		dead_letters: Default::default(),
		endpoint: "https://streams.gameinc.io".into(),
		token: None,
		primary: false,
		subscriptions: BTreeMap::from([(
			"orders".into(),
			NamedStreamSubscription {
				stream: "/orders/:id".into(),
				actor_key: vec![":id".into()],
			},
		)]),
	};
	config.validate().unwrap();
	let exported = config.declarations("customer");
	assert!(!exported.contains_key("primary"));
	assert_eq!(
		serde_json::to_value(&exported["orders"]).unwrap()["destination"]["actorKey"]["actorName"],
		"customer"
	);
	config
		.subscriptions
		.insert("primary".into(), config.subscriptions["orders"].clone());
	assert!(config.validate().is_err());
}
