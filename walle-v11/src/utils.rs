use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Status {
    pub online: bool,
    pub good: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Sender {
    Private(PrivateSender),
    Group(GroupSender),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PrivateSender {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GroupSender {
    pub user_id: i64,
    pub nickname: String,
    pub card: String,
    pub sex: String,
    pub age: i32,
    pub area: String,
    pub level: String,
    pub role: String,
    pub title: String,
}
