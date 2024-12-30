use chirp_workflow::prelude::*;
use proto::backend::{self};
use rivet_operation::prelude::{proto};


#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
struct CacheUserIdentity {
	user_id: Uuid,
	email: Option<String>,
}

#[derive(Debug)]
pub struct Input {
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug)]
pub struct Output {
    pub users: Vec<User>
}

#[derive(Debug)]
pub struct User {
    pub user_id: Uuid,
    pub identities: Vec<backend::user_identity::Identity>
}


#[operation]
pub async fn get(
    ctx: &OperationCtx,
    input: &Input
) -> GlobalResult<Output> {
    let is_development = ctx.config().server()?.rivet.auth.access_kind
        == rivet_config::config::rivet::AccessKind::Development;

    let user_ids = &input.user_ids;
    // Get the user display names
    let users = ctx.op(crate::ops::get::Input {
        user_ids: user_ids.clone(),
    })
    .await?;

    // Fetch the identities
    let identities = ctx
        .cache()
        .fetch_all_json("user_identity.identity", user_ids.clone(), {
            let ctx = ctx.clone();
            move |mut cache, user_ids| {
                let ctx = ctx.clone();
                async move {
                    let identity_rows = sql_fetch_all!(
                        [ctx, CacheUserIdentity]
                        "
                        SELECT e.user_id AS user_id, e.email
                        FROM db_user_identity.emails as e
                        WHERE e.user_id = ANY($1)
                        ",
                        &user_ids,
                    )
                    .await?;

                    for row in identity_rows {
                        cache.resolve(
                            &row.user_id.clone(),
                            row,
                        );
                    }

                    Ok(cache)
                }
            }
        })
        .await?;

    let users = user_ids
        .iter()
        .filter_map(|user_id| {
            // Find matching user
            let Some(user) = users
                .users
                .iter()
                .find(|x| x.user_id.map(|x| x.as_uuid()) == Some(*user_id))
            else {
                return None;
            };

            // Find matching identities
            let mut identities = identities
                .iter()
                .filter(|x| x.user_id == *user_id)
                .flat_map(|x| {
                    IntoIterator::into_iter([x.email.as_ref().map(|email| {
                        backend::user_identity::Identity {
                            kind: Some(backend::user_identity::identity::Kind::Email(
                                backend::user_identity::identity::Email {
                                    email: email.clone(),
                                },
                            )),
                        }
                    })])
                    .flatten()
                })
                .collect::<Vec<_>>();

            // Inject identity for development users since they should behave like registered users.
            if is_development && user.display_name == util::dev_defaults::USER_NAME {
                identities.push(backend::user_identity::Identity {
                    kind: Some(backend::user_identity::identity::Kind::DefaultUser(
                        backend::user_identity::identity::DefaultUser {},
                    )),
                })
            }

            Some(User {
                user_id: *user_id,
                identities,
            })
        })
        .collect::<Vec<_>>();

    Ok(Output { users })
}
