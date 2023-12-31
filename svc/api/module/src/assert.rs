use rivet_operation::prelude::*;

/// Validates that a given user ID is registered.
pub async fn user_registered(ctx: &OperationContext<()>, user_id: Uuid) -> GlobalResult<()> {
	// If the user has at least one identity they are considered registered
	let identity = op!([ctx] user_identity_get {
		user_ids: vec![user_id.into()]
	})
	.await?;

	let identities = &unwrap_ref!(identity.users.first()).identities;
	ensure_with!(!identities.is_empty(), IDENTITY_NOT_REGISTERED);

	Ok(())
}
