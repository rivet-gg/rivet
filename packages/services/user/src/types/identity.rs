#[derive(Clone, PartialEq)]
pub struct Email {
    pub email: String,
}

#[derive(Clone, PartialEq)]
pub struct DefaultUser {}

#[derive(Clone, PartialEq)]
pub enum Kind {
    Email(Email),
    DefaultUser(DefaultUser),
}

#[derive(Clone, PartialEq)]
pub struct Identity {
    pub kind: Kind
}