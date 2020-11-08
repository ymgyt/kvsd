use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::common::{error, info, ErrorKind, Result};
use crate::core::{principal, Config, Credential, Password, Principal, UnitOfWork, UserEntry};

#[derive(Default)]
pub(crate) struct Builder {
    config: Option<Config>,
    request_channel_buffer: usize,
}

impl Builder {
    pub(crate) fn from_config(config: Config) -> Self {
        let mut builder = Builder::new();
        builder.config = Some(config);
        builder
    }

    pub(crate) fn build(self) -> Result<Kvs> {
        let (send, recv) = mpsc::channel(self.request_channel_buffer);
        Ok(Kvs {
            request_send: send,
            request_recv: recv,
            users: self.config.unwrap_or_default().users,
        })
    }

    fn new() -> Self {
        Self {
            request_channel_buffer: 1024,
            ..Default::default()
        }
    }
}

pub(crate) struct Kvs {
    request_recv: Receiver<UnitOfWork>,
    request_send: Sender<UnitOfWork>,
    users: Vec<UserEntry>,
}

impl Kvs {
    pub fn request_channel(&self) -> Sender<UnitOfWork> {
        self.request_send.clone()
    }

    pub(crate) async fn run(mut self) {
        info!("Kvs running");

        while let Some(request) = self.request_recv.recv().await {
            // TODO: middleware, dispatcher
            if let Err(err) = self.handle_request(request).await {
                error!("Handle request {}", err);
            }
        }
    }

    pub(crate) async fn handle_request(&mut self, uow: UnitOfWork) -> Result<()> {
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
            }
            UnitOfWork::Ping(ping) => {
                use chrono::Utc;
                // mock network latency.
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    100 + (rand::random::<f64>() * 100.0) as u64,
                ))
                .await;

                // TODO: handle unauthenticated error
                assert!(ping.principal.is_authenticated());
                info!(user=?ping.principal, "Ping");
                let response = if !ping.principal.is_authenticated() {
                    Err(ErrorKind::Unauthorized("unauthorized ping".to_owned()).into())
                } else {
                    Ok(Utc::now())
                };

                ping.response_sender
                    .send(response)
                    .map_err(|_| ErrorKind::Internal("send to resp channel".to_owned()))?;
            }
        }

        Ok(())
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
