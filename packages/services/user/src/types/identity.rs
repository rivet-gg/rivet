use rivet_operation::prelude::proto::backend;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Hash)]
pub struct Email {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Hash)]
pub struct DefaultUser {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Hash)]
pub enum Kind {
    Email(Email),
    DefaultUser(DefaultUser),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Hash)]
pub struct Identity {
    pub kind: Kind
}

impl From<&Identity> for backend::user_identity::Identity {
    fn from(identity: &Identity) -> Self {
        backend::user_identity::Identity {
            kind: Some(match &identity.kind {
                Kind::Email(kind) => {
                    backend::user_identity::identity::Kind::Email(
                        backend::user_identity::identity::Email {
                            email: kind.email.clone()
                        }
                    )
                }
                Kind::DefaultUser(_) => {
                    backend::user_identity::identity::Kind::DefaultUser(
                        backend::user_identity::identity::DefaultUser {}
                    )
                }
            })
        }
    }
}