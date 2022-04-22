use walle_core::MessageContent;

use super::{start_with, strip_prefix};
use crate::{Handler, HandlerExt};

pub fn on_command<H>(command: &str, handler: H) -> impl Handler<MessageContent>
where
    H: Handler<MessageContent> + Sync,
{
    handler
        .rule(start_with(command))
        .pre_handle(strip_prefix(command))
}
