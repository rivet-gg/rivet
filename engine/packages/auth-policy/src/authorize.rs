use crate::{AccessRequest, Grant, OperationKind, OwnedGrant, ResourceKind, Scope};

/// Signing endpoints have no caller-selected namespace. Only concrete grants for the
/// requested operation can select it; unrelated and wildcard grants do not select one.
pub fn signing_namespace<NamespaceId: Copy + Eq, TargetId>(
	grants: &[OwnedGrant<NamespaceId, TargetId>],
	resource: ResourceKind,
) -> Option<NamespaceId> {
	if !matches!(
		resource,
		ResourceKind::ApplicationToken | ResourceKind::WebhookSigning
	) {
		return None;
	}
	let mut namespace = None;
	for grant in grants {
		if grant.resource != resource
			|| !grant.operations.contains(&OperationKind::Create)
			|| !matches!(grant.target, Scope::Any)
		{
			continue;
		}
		let Scope::Id(candidate) = grant.namespace else {
			continue;
		};
		if namespace.is_some_and(|existing| existing != candidate) {
			return None;
		}
		namespace = Some(candidate);
	}
	namespace
}

/// `Scope::Any` widens only the grant side of the comparison.
pub fn grant_allows<NamespaceId: PartialEq, TargetId: PartialEq>(
	grant: &Grant<'_, NamespaceId, TargetId>,
	request: &AccessRequest<NamespaceId, TargetId>,
) -> bool {
	grant.resource == request.resource
		&& grant.operations.contains(&request.operation)
		&& scope_matches(&grant.namespace, &request.namespace)
		&& scope_matches(&grant.target, &request.target)
}

pub fn is_authorized<'a, NamespaceId: PartialEq, TargetId: PartialEq>(
	request: &AccessRequest<NamespaceId, TargetId>,
	grants: impl IntoIterator<Item = Grant<'a, NamespaceId, TargetId>>,
) -> bool {
	grants
		.into_iter()
		.any(|grant| grant_allows(&grant, request))
}

/// This prevents delegation from widening namespace, resource, target, or operation authority.
pub fn can_delegate<NamespaceId: PartialEq, TargetId: PartialEq>(
	parent: &[OwnedGrant<NamespaceId, TargetId>],
	requested: &[OwnedGrant<NamespaceId, TargetId>],
) -> bool {
	requested.iter().all(|grant| {
		grant.operations.iter().all(|&operation| {
			let request = AccessRequest {
				namespace: grant.namespace.as_ref(),
				resource: grant.resource,
				target: grant.target.as_ref(),
				operation,
			};
			is_authorized(&request, parent.iter().map(OwnedGrant::as_grant))
		})
	})
}
fn scope_matches<T: PartialEq>(grant: &Scope<T>, request: &Scope<T>) -> bool {
	match (grant, request) {
		(Scope::Any, _) => true,
		(Scope::Id(grant_id), Scope::Id(request_id)) => grant_id == request_id,
		(Scope::Id(_), Scope::Any) => false,
	}
}
