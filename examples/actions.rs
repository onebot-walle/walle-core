use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Action0 {
    pub param0: String,
    pub param1: u8,
}

pub struct Action1 {
    pub param0: String,
    pub param1: u8,
}

use walle_core::{onebot_action, onebot_action_ext, ExtendedMap};
onebot_action!(ActionExtA, field0: u8);
onebot_action_ext!(MyActionExt => ActionExtA);

fn main() {}
