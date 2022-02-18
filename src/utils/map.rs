use serde::{de::Visitor, Deserialize, Serialize};
use std::collections::HashMap;

use crate::WalleError;

/// 扩展字段 Map
pub type ExtendedMap = HashMap<String, ExtendedValue>;

/// 扩展字段 MapValue
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedValue {
    Str(String),
    F64(f64),
    Int(i64),
    Bool(bool),
    Map(HashMap<String, ExtendedValue>),
    List(Vec<ExtendedValue>),
    #[serde(serialize_with = "null_serialize")]
    // deserialize_with = "null_deserialize" will cause error
    Null,
}

macro_rules! impl_from {
    ($inty: tt,$ty: ty) => {
        impl From<$ty> for ExtendedValue {
            fn from(v: $ty) -> Self {
                ExtendedValue::$inty(v)
            }
        }
    };
    ($inty: tt, $ty: ty, $ty0:ty) => {
        impl From<$ty> for ExtendedValue {
            fn from(v: $ty) -> Self {
                ExtendedValue::$inty(v as $ty0)
            }
        }
    };
}

impl_from!(Str, String);
impl_from!(Int, i64);
impl_from!(Int, i8, i64);
impl_from!(Int, i16, i64);
impl_from!(Int, i32, i64);
impl_from!(F64, f64);
impl_from!(F64, f32, f64);
impl_from!(Bool, bool);

impl From<&str> for ExtendedValue {
    fn from(v: &str) -> Self {
        ExtendedValue::Str(v.to_owned())
    }
}

impl<T> From<HashMap<String, T>> for ExtendedValue
where
    T: Into<ExtendedValue>,
{
    fn from(v: HashMap<String, T>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in v {
            map.insert(k, v.into());
        }
        ExtendedValue::Map(map)
    }
}

impl<T> From<Vec<T>> for ExtendedValue
where
    T: Into<ExtendedValue>,
{
    fn from(v: Vec<T>) -> Self {
        let mut list = Vec::new();
        for v in v {
            list.push(v.into());
        }
        ExtendedValue::List(list)
    }
}

macro_rules! impl_try_from {
    ($inty: tt, $ty: ty) => {
        impl TryFrom<ExtendedValue> for $ty {
            type Error = ExtendedValue;

            fn try_from(i: ExtendedValue) -> Result<$ty, Self::Error> {
                match i {
                    ExtendedValue::$inty(v) => Ok(v),
                    _ => Err(i),
                }
            }
        }
    };
}

impl_try_from!(Str, String);
impl_try_from!(Int, i64);
impl_try_from!(F64, f64);
impl_try_from!(Bool, bool);
impl_try_from!(Map, ExtendedMap);
impl_try_from!(List, Vec<ExtendedValue>);

fn null_serialize<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_none()
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = ExtendedValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("accepts string, f64, int, bool, map, list, null")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(ExtendedValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(ExtendedValue::Int(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(ExtendedValue::Int(v as i64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(ExtendedValue::F64(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtendedValue::Str(v.to_owned()))
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(ExtendedValue::List(values))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::MapAccess<'de>,
    {
        let mut map = HashMap::new();
        while let Some((key, value)) = visitor.next_entry()? {
            map.insert(key, value);
        }
        Ok(ExtendedValue::Map(map))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtendedValue::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtendedValue::Null)
    }
}

impl<'de> Deserialize<'de> for ExtendedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

macro_rules! downcast_fn {
    ($fname: ident, $ty: ty) => {
        pub fn $fname(self) -> Result<$ty, Self> {
            self.downcast()
        }
    };
}

#[allow(dead_code)]
impl ExtendedValue {
    pub fn empty() -> Self {
        Self::Map(ExtendedMap::default())
    }

    pub fn downcast<T>(self) -> Result<T, Self>
    where
        T: TryFrom<ExtendedValue, Error = Self>,
    {
        self.try_into()
    }

    downcast_fn!(downcast_str, String);
    downcast_fn!(downcast_int, i64);
    downcast_fn!(downcast_f64, f64);
    downcast_fn!(downcast_bool, bool);
    downcast_fn!(downcast_map, ExtendedMap);
    downcast_fn!(downcast_list, Vec<ExtendedValue>);

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }
    pub fn is_f64(&self) -> bool {
        matches!(self, Self::F64(_))
    }
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

pub trait ExtendedMapExt {
    fn try_remove<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = ExtendedValue>;
}

impl ExtendedMapExt for ExtendedMap {
    fn try_remove<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = ExtendedValue>,
    {
        let value = self
            .remove(key)
            .ok_or_else(|| WalleError::MapMissedKey(key.to_owned()))?;
        T::try_from(value).map_err(|v| {
            let msg = format!("{:?}", v);
            self.insert(key.to_owned(), v);
            WalleError::MapValueTypeMismatch(std::any::type_name::<T>().to_string(), msg)
        })
    }
}
