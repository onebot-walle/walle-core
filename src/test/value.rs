use crate::{
    prelude::WalleError,
    util::{PushToValueMap, Value, ValueMap, ValueMapExt},
};

pub struct ValueStruct {
    pub field0: u32,
    pub field1: String,
}

impl TryFrom<&ValueMap> for ValueStruct {
    type Error = WalleError;
    fn try_from(map: &ValueMap) -> Result<Self, Self::Error> {
        Ok(Self {
            field0: map.get_downcast("field0")?,
            field1: map.get_downcast("field1")?,
        })
    }
}

impl TryFrom<&Value> for ValueStruct {
    type Error = WalleError;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(map) => Self::try_from(map),
            _ => Err(WalleError::DeclareNotMatch(
                "Struct Value",
                "not struct".to_owned(),
            )),
        }
    }
}

impl TryFrom<&mut ValueMap> for ValueStruct {
    type Error = WalleError;
    fn try_from(map: &mut ValueMap) -> Result<Self, Self::Error> {
        Ok(Self {
            field0: map.remove_downcast("field0")?,
            field1: map.remove_downcast("field1")?,
        })
    }
}

impl TryFrom<&mut Value> for ValueStruct {
    type Error = WalleError;
    fn try_from(value: &mut Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(map) => Self::try_from(map),
            _ => Err(WalleError::DeclareNotMatch(
                "Struct Value",
                "not struct".to_owned(),
            )),
        }
    }
}

impl PushToValueMap for ValueStruct {
    fn push_to(self, map: &mut ValueMap)
    where
        Self: Sized,
    {
        map.insert("field0".to_owned(), self.field0.into());
        map.insert("field1".to_owned(), self.field1.into());
    }
}

impl From<ValueStruct> for ValueMap {
    fn from(s: ValueStruct) -> Self {
        let mut map = ValueMap::default();
        s.push_to(&mut map);
        map
    }
}

impl From<ValueStruct> for Value {
    fn from(s: ValueStruct) -> Self {
        Value::Map(s.into())
    }
}

