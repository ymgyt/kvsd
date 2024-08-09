mod user;
pub(crate) use user::User;

#[derive(Debug, Clone)]
pub(crate) enum Principal {
    AnonymousUser,
    #[allow(dead_code)]
    User(User),
}

impl Principal {
    pub(crate) fn is_authenticated(&self) -> bool {
        matches!(self, Principal::User(_))
    }
}
