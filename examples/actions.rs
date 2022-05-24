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

use walle_core::{extra_struct, onebot_action_ext, ExtendedMap};
extra_struct!(ActionExtA, field0: u8);
onebot_action_ext!(MyActionExt => ActionExtA);

fn main() {}
