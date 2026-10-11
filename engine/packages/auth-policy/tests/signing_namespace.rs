use rivet_auth_policy::{
	OperationKind as Op, OwnedGrant, ResourceKind as Resource, Scope, signing_namespace,
};

fn grant(namespace: Scope<u32>, resource: Resource) -> OwnedGrant<u32, u32> {
	OwnedGrant {
		namespace,
		resource,
		target: Scope::Any,
		operations: vec![Op::Create],
	}
}

#[test]
fn signing_requires_one_concrete_namespace_for_the_requested_permission() {
	let application = Resource::ApplicationToken;
	let webhook = Resource::WebhookSigning;
	let one = grant(Scope::Id(1), application);
	let wildcard = grant(Scope::Any, application);
	assert_eq!(signing_namespace::<u32, u32>(&[], application), None);
	assert_eq!(signing_namespace(&[wildcard.clone()], application), None);
	assert_eq!(
		signing_namespace(&[one.clone(), one.clone(), wildcard], application),
		Some(1)
	);
	assert_eq!(
		signing_namespace(
			&[one.clone(), grant(Scope::Id(2), application)],
			application
		),
		None
	);
	assert_eq!(
		signing_namespace(&[one.clone(), grant(Scope::Id(2), webhook)], application),
		Some(1)
	);
	assert_eq!(signing_namespace(&[one.clone()], webhook), None);
	assert_eq!(
		signing_namespace(&[grant(Scope::Id(1), Resource::Jwt)], Resource::Jwt),
		None
	);
	let mut wrong_target = one.clone();
	wrong_target.target = Scope::Id(1);
	assert_eq!(signing_namespace(&[wrong_target], application), None);
	let mut wrong_operation = one;
	wrong_operation.operations = vec![Op::Read];
	assert_eq!(signing_namespace(&[wrong_operation], application), None);
}
