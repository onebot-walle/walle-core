use crate::{Action, Event, Resp};
pub type OneBot11<H> = walle_core::impls::CustomOneBot<Event, Action, Resp, H, 11>;

// impl<A, R, const V: u8> walle_core::impls::CustomOneBot<Event, A, R, V> {}
