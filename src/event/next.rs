use crate::util::ExtendedMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub detail_type: String,
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}
