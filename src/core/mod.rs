//! A module does not depends on communication processes such as tcp.
//!
//! The key value management feature of kvsd is intended to be able to be embed directory into the application.

mod kvsd;
pub(crate) use self::kvsd::Builder;

mod config;
pub use self::config::{Config, UserEntry};

mod table;
pub(crate) use table::{EntryDump, Table};

mod principal;
pub(crate) use self::principal::Principal;

pub(crate) mod uow;
pub(crate) use self::uow::{UnitOfWork, Work};

mod credential;
pub(crate) use self::credential::{Credential, Password, Provider as CredentialProvider};

mod middleware;
