use super::*;

fn manifest(version: u32, names: &[&str]) -> Manifest {
	Manifest {
		supports_stream_subscriptions: true,
		version,
		rivetkit_version: Some("2.3.20".into()),
		actor_names: names
			.iter()
			.map(|name| ((*name).into(), Map::new()))
			.collect(),
	}
}

#[test]
fn deployments_are_monotonic_and_same_version_is_immutable() {
	let old = manifest(3, &["customer"]);
	assert!(should_publish(None, &old).unwrap());
	assert!(!should_publish(Some(&old), &old).unwrap());
	assert!(!should_publish(Some(&old), &manifest(2, &[])).unwrap());
	assert!(should_publish(Some(&old), &manifest(3, &[])).is_err());
	assert!(should_publish(Some(&old), &manifest(4, &[])).unwrap());
}

#[test]
fn cross_datacenter_removals_win_regardless_of_response_order() {
	let active =
		json!({"streamSubscriptionSources":{"pool":{"version":3,"subscriptions":{"orders":{}}}}});
	let removed = json!({"streamSubscriptionSources":{"pool":{"version":4,"subscriptions":{}}}});
	for (first, second) in [(&active, &removed), (&removed, &active)] {
		let mut target = first.as_object().unwrap().clone();
		merge_sources(&mut target, second.as_object().unwrap());
		assert_eq!(Value::Object(target), removed);
	}
}

#[test]
fn equal_version_conflicts_fail_closed() {
	let mut first =
		json!({"streamSubscriptionSources":{"pool":{"version":3,"subscriptions":{"orders":{}}}}})
			.as_object()
			.unwrap()
			.clone();
	let second =
		json!({"streamSubscriptionSources":{"pool":{"version":3,"subscriptions":{"other":{}}}}});
	merge_sources(&mut first, second.as_object().unwrap());
	assert_eq!(
		first["streamSubscriptionSources"]["pool"]["subscriptions"],
		json!({})
	);
	assert_eq!(first["streamSubscriptionSources"]["pool"]["conflict"], true);
}

#[test]
fn legacy_publishers_cannot_replace_or_invent_owned_sources() {
	let existing =
		json!({"streamSubscriptionSources":{"pool":{"version":4,"subscriptions":{}}},"icon":"old"})
			.as_object()
			.unwrap()
			.clone();
	let incoming=json!({"streamSubscriptionSources":{"pool":{"version":99,"subscriptions":{"bad":{}}}},"icon":"new"}).as_object().unwrap().clone();
	let merged = legacy_metadata(Some(existing), incoming.clone());
	assert_eq!(merged["icon"], "new");
	assert_eq!(merged["streamSubscriptionSources"]["pool"]["version"], 4);
	assert!(
		legacy_metadata(None, incoming)
			.get("streamSubscriptionSources")
			.is_none()
	);
}
