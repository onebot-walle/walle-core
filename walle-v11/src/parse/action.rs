use crate::action::{Action as V11Action, Resp as V11Resp};
use walle_core::{Action as V12Action, Resps as V12Resps};

impl TryFrom<V12Action> for V11Action {
    type Error = super::WalleParseError;
    fn try_from(value: V12Action) -> Result<Self, Self::Error> {
        match value {
            _ => todo!(),
        }
    }
}

impl TryInto<V12Action> for V11Action {
    type Error = super::WalleParseError;
    fn try_into(self) -> Result<V12Action, Self::Error> {
        todo!();
    }
}

impl TryFrom<V12Resps> for V11Resp {
    type Error = super::WalleParseError;
    fn try_from(value: V12Resps) -> Result<Self, Self::Error> {
        match value {
            _ => todo!(),
        }
    }
} 

impl TryInto<V12Resps> for V11Resp {
    type Error = super::WalleParseError;
    fn try_into(self) -> Result<V12Resps, Self::Error> {
        todo!();
    }
}