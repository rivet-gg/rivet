use rivet_api::models;
use rivet_operation::prelude::*;

use crate::ApiTryFrom;

impl ApiTryFrom<user::types::identity::Identity> for models::IdentityLinkedAccount {
	type Error = GlobalError;

	fn api_try_from(
		value: user::types::identity::Identity,
	) -> GlobalResult<models::IdentityLinkedAccount> {
		match value.kind {
			user::types::identity::Kind::Email(email_ident) => {
				Ok(models::IdentityLinkedAccount {
					email: Some(Box::new(models::IdentityEmailLinkedAccount {
						email: email_ident.email.to_owned(),
					})),
					..Default::default()
				})
			}
			user::types::identity::Kind::DefaultUser(_) => {
				Ok(models::IdentityLinkedAccount {
					default_user: Some(true),
					..Default::default()
				})
			}
		}
	}
}
