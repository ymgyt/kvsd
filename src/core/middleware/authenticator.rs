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

    fn check_principal(&self, principal: &Principal) -> Result<()> {
        if principal.is_authenticated() {
            Ok(())
        } else {
            Err(ErrorKind::Unauthenticated.into())
        }
    }
}

#[async_trait]
impl<MW> Middleware for Authenticator<MW>
where
    MW: Middleware + Send + 'static,
{
    async fn apply(&mut self, uow: UnitOfWork) -> Result<()> {
        // TODO: need derive impl that can provide principal
        match uow {
            UnitOfWork::Authenticate(auth) => {
                let credential = auth.request.credential();
                match credential {
                    Credential::Password(password) => {
                        info!(user=?password.username, "Try authenticate ");
                        auth.response_sender
                            .expect("response already sent")
                            .send(self.authenticate_by_password(password))
                            .map_err(|_| ErrorKind::Internal("send to resp channel".into()))?;
                    }
                }

                Ok(())
            }
            UnitOfWork::Ping(Work { ref principal, .. })
            | UnitOfWork::Set(Work { ref principal, .. })
            | UnitOfWork::Get(Work { ref principal, .. }) => {
                let r = self.check_principal(principal.as_ref());

                match r {
                    Ok(_) => self.next.apply(uow).await,
                    Err(err) => Err(err),
                }
            }
        }
    }
}
