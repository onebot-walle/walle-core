use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::event::Event;

#[async_trait]
pub trait KVDB {
    async fn new() -> Self;
    async fn get_event(&self, id: &str) -> Option<Event>;
    async fn insert_event(&self, event: &Event) -> Option<Event>;
    async fn get_file<T: for<'de> Deserialize<'de>>(&self, file_id: &str) -> Option<T>;
    async fn insert_file<T: Serialize>(&self, file_id: &str, file: &T) -> Option<T>;
}

#[async_trait]
pub trait SqlDB: KVDB {
    // todo!
}
