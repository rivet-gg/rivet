use chirp_workflow::prelude::*;
use email_address_parser::EmailAddress;
use rivet_operation::prelude::proto;
use proto::backend::{self,pkg::*};
use serde_json::json;

#[derive(Debug)]
pub struct Input {
    pub user_id: Uuid,
    pub identity: backend::user_identity::Identity,
}

#[derive(Debug)]
pub struct Output {}


#[operation]
pub async fn create(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
    let user_id = input.user_id;
	let identity = &input.identity;
	let identity_kind = unwrap_ref!(identity.kind);

	match &identity_kind {
		backend::user_identity::identity::Kind::Email(email) => {
			ensure!(EmailAddress::is_valid(&email.email, None), "invalid email");

			sql_execute!(
				[ctx]
				"
				INSERT INTO db_user_identity.emails (email, user_id, create_ts)
				VALUES ($1, $2, $3)
				",
				&email.email,
				user_id,
				ctx.ts(),
			)
			.await?;

			msg!([ctx] analytics::msg::event_create() {
				events: vec![
					analytics::msg::event_create::Event {
						event_id: Some(Uuid::new_v4().into()),
						name: "user_identity.create".into(),
						properties_json: Some(serde_json::to_string(&json!({
							"identity_email": email.email,
							"user_id": user_id,
						}))?),
						..Default::default()
					}
				],
			})
			.await?;
		}
		backend::user_identity::identity::Kind::DefaultUser(_) => {
			bail!("cannot create default user identity")
		}
	}

	ctx.cache()
		.purge("user_identity.identity", [user_id])
		.await?;

	msg!([ctx] user_identity::msg::create_complete(user_id) {
		user_id: Some(user_id.into()),
		identity: Some(identity.clone()),
	})
	.await?;

	msg!([ctx] user::msg::update(user_id) {
		user_id: Some(user_id.into()),
	})
	.await?;

	Ok(Output {})
}