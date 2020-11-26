use async_trait::async_trait;

use crate::{Key, Result, Value};

pub mod tcp;

#[async_trait]
pub trait Api {
    async fn ping(&mut self) -> Result<chrono::Duration>;
    async fn set(&mut self, key: Key, value: Value) -> Result<()>;
    async fn get(&mut self, key: Key) -> Result<Option<Value>>;
    async fn delete(&mut self, key: Key) -> Result<Option<Value>>;
}
