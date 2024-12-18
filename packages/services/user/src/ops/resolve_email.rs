use chirp_workflow::prelude::*;

#[derive(Debug)]
pub struct Input {
	pub emails: Vec<String>,
}

#[derive(Debug)]
pub struct Output {
	pub users: Vec<User>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User {
	pub email: String,
	pub user_id: Uuid,
}

#[operation]
pub async fn resolve_email(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
	let users = sql_fetch_all!(
		[ctx, User]
		"
		SELECT email, user_id
		FROM db_user_identity.emails
		WHERE email = ANY($1)
	",
		&input.emails,
	)
	.await?
	.into_iter()
	.map(|row| User {
		email: row.email,
		user_id: row.user_id,
	})
	.collect::<Vec<_>>();

	Ok(Output { users })
}
