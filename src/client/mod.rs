//! Provides an implementation of kvsd protocol communication with the kvsd server.

use async_trait::async_trait;

use crate::{Key, Result, Value};

/// tcp client implementation.
pub mod tcp;

/// Kvsd Api.
#[async_trait]
pub trait Api {
    /// Ping to server.
    async fn ping(&mut self) -> Result<chrono::Duration>;

    /// Set given key value to remote kvsd.
    async fn set(&mut self, key: Key, value: Value) -> Result<()>;

    /// Get the value corresponding to the key.
    async fn get(&mut self, key: Key) -> Result<Option<Value>>;

    /// Delete the value corresponding to the key.
    /// if the key exists, return the deleted value.
    async fn delete(&mut self, key: Key) -> Result<Option<Value>>;
}
