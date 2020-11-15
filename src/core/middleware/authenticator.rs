use async_trait::async_trait;

use crate::common::{info, ErrorKind, Result};
use crate::core::middleware::Middleware;
use crate::core::{principal, Credential, Password, Principal, UnitOfWork, UserEntry, Work};

pub(crate) struct Authenticator<MW> {
    users: Vec<UserEntry>,
    next: MW,
}

impl<MW> Authenticator<MW> {
    pub(crate) fn new(users: Vec<UserEntry>, next: MW) -> Self {
        Self { users, next }
    }

    fn authenticate_by_password(&self, password: Password) -> Result<Option<Principal>> {
        for user_entry in &self.users {
            if user_entry.username == password.username && user_entry.password == password.password
            {
                return Ok(Some(Principal::User(principal::User {
                    name: user_entry.username.clone(),
                })));
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl<MW> Middleware for Authenticator<MW>
where
    MW: Middleware + Send + 'static,
{
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {
        match uow {
            UnitOfWork::Authenticate(auth) => {
                let credential = auth.request.credential();
                match credential {
                    Credential::Password(password) => {
                        info!(user=?password.username, "Try authenticate ");
                        auth.response_sender
                            .send(self.authenticate_by_password(password))
                            .map_err(|_| ErrorKind::Internal("send to resp channel".into()))?;
                    }
                }

                Ok(())
            }
            UnitOfWork::Ping(Work { ref principal, .. }) => {
                if !principal.is_authenticated() {
                    // TODO: write unauthenticated response.
                    todo!()
                } else {
                    self.next.apply(uow).await
                }
            }
        }
    }
}
