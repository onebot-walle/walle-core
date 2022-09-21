use crate::{
    prelude::WalleError,
    util::{PushToValueMap, Value, ValueMap, ValueMapExt},
};

pub struct NamedValueStruct {
    pub field0: u32,
    pub field1: String,
}

//From
impl TryFrom<&mut ValueMap> for NamedValueStruct {
    type Error = WalleError;
    fn try_from(map: &mut ValueMap) -> Result<Self, Self::Error> {
        Ok(Self {
            field0: map.remove_downcast("field0")?,
            field1: map.remove_downcast("field1")?,
        })
    }
}

impl TryFrom<Value> for NamedValueStruct {
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(mut map) => Self::try_from(&mut map),
            v => Err(WalleError::DeclareNotMatch("Map Value", format!("{:?}", v))),
        }
    }
}

//Push
impl PushToValueMap for NamedValueStruct {
    fn push_to(self, map: &mut ValueMap)
    where
        Self: Sized,
    {
        map.insert("field0".to_owned(), self.field0.into());
        map.insert("field1".to_owned(), self.field1.into());
    }
}

impl From<NamedValueStruct> for ValueMap {
    fn from(s: NamedValueStruct) -> Self {
        let mut map = ValueMap::default();
        s.push_to(&mut map);
        map
    }
}

impl From<NamedValueStruct> for Value {
    fn from(s: NamedValueStruct) -> Self {
        Value::Map(s.into())
    }
}

pub struct UnnamedValueStruct(NamedValueStruct);

impl TryFrom<&mut ValueMap> for UnnamedValueStruct {
    type Error = WalleError;
    fn try_from(map: &mut ValueMap) -> Result<Self, Self::Error> {
        //mut
        Ok(Self(NamedValueStruct::try_from(map)?))
    }
}

impl TryFrom<&mut Value> for UnnamedValueStruct {
    type Error = WalleError;
    fn try_from(value: &mut Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(map) => Self::try_from(map),
            v => Err(WalleError::DeclareNotMatch("Map Value", format!("{:?}", v))),
        }
    }
}

impl PushToValueMap for UnnamedValueStruct {
    fn push_to(self, map: &mut ValueMap)
    where
        Self: Sized,
    {
        //mut
        self.0.push_to(map);
    }
}

impl From<UnnamedValueStruct> for ValueMap {
    fn from(s: UnnamedValueStruct) -> Self {
        let mut map = ValueMap::default();
        s.push_to(&mut map);
        map
    }
}

impl From<UnnamedValueStruct> for Value {
    fn from(s: UnnamedValueStruct) -> Self {
        Value::Map(s.into())
    }
}

// unsupport
// pub enum ValueEnum {
//     Enum0,
//     Enum1(u8),
//     Enum2 { field: u8 },
// }

use walle_macro::_TryFromValue as TryFromValue;

#[derive(TryFromValue)]
pub struct TestStruct;

#[derive(TryFromValue)]
pub struct TestStruct2 {
    pub f0: TestStruct,
}
