use std::collections::HashMap;

use super::OneBotBytes;
use crate::error::{WalleError, WalleResult};

/// 扩展字段 Map
pub type ValueMap = HashMap<String, Value>;

mod _macro;
mod _serde;

/// 扩展字段 MapValue
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    F64(f64),
    Int(i64),
    Bool(bool),
    Map(ValueMap),
    List(Vec<Value>),
    Bytes(OneBotBytes),
    Null,
}

macro_rules! impl_from {
    ($inty: tt,$ty: ty) => {
        impl From<$ty> for Value {
            fn from(v: $ty) -> Self {
                Value::$inty(v)
            }
        }
    };
    ($inty: tt, $ty: ty, $ty0:ty) => {
        impl From<$ty> for Value {
            fn from(v: $ty) -> Self {
                Value::$inty(v as $ty0)
            }
        }
    };
}

impl_from!(Str, String);
impl_from!(Int, i64);
impl_from!(Int, i8, i64);
impl_from!(Int, i16, i64);
impl_from!(Int, i32, i64);
// impl_from!(Int, u8, i64);
impl_from!(Int, u16, i64);
impl_from!(Int, u32, i64);
impl_from!(F64, f64);
impl_from!(F64, f32, f64);
impl_from!(Bool, bool);
impl_from!(Bytes, OneBotBytes);

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bytes(OneBotBytes(v))
    }
}

impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Bytes(OneBotBytes(v.to_vec()))
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Str(v.to_owned())
    }
}

impl<T> From<HashMap<String, T>> for Value
where
    T: Into<Value>,
{
    fn from(v: HashMap<String, T>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in v {
            map.insert(k, v.into());
        }
        Value::Map(map)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Vec<T>) -> Self {
        let mut list = Vec::new();
        for v in v {
            list.push(v.into());
        }
        Value::List(list)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

macro_rules! impl_try_from {
    ($inty: tt, $ty: ty) => {
        impl TryFrom<Value> for $ty {
            type Error = WalleError;
            fn try_from(i: Value) -> Result<$ty, Self::Error> {
                match i {
                    Value::$inty(v) => Ok(v),
                    v => Err(WalleError::ValueTypeNotMatch(
                        std::any::type_name::<$ty>().to_string(),
                        format!("{:?}", v),
                    )),
                }
            }
        }
    };
    ($inty: tt as $ty: ty) => {
        impl TryFrom<Value> for $ty {
            type Error = WalleError;
            fn try_from(i: Value) -> Result<$ty, Self::Error> {
                match i {
                    Value::$inty(v) => Ok(v as $ty),
                    v => Err(WalleError::ValueTypeNotMatch(
                        std::any::type_name::<$ty>().to_string(),
                        format!("{:?}", v),
                    )),
                }
            }
        }
    };
}

impl_try_from!(Str, String);
impl_try_from!(Int, i64);
impl_try_from!(Int as i32);
impl_try_from!(Int as u32);
impl_try_from!(Int as i16);
impl_try_from!(Int as u16);
impl_try_from!(Int as i8);
impl_try_from!(Int as u8);
impl_try_from!(F64, f64);
impl_try_from!(F64 as f32);
impl_try_from!(Bool, bool);

impl<V> TryFrom<Value> for Vec<V>
where
    V: TryFrom<Value, Error = WalleError>,
{
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::List(l) => l.into_iter().map(|v| v.try_into()).collect(),
            v => Err(WalleError::ValueTypeNotMatch(
                format!("List<{}>", std::any::type_name::<V>()),
                format!("{:?}", v),
            )),
        }
    }
}

impl<V> TryFrom<Value> for HashMap<String, V>
where
    V: TryFrom<Value, Error = WalleError>,
{
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Map(m) => m
                .into_iter()
                .map(|e| e.1.try_into().map(|v| (e.0, v)))
                .collect(),
            v => Err(WalleError::ValueTypeNotMatch(
                format!("Map<{}>", std::any::type_name::<V>()),
                format!("{:?}", v),
            )),
        }
    }
}

// impl<T> TryFrom<ExtendedValue> for Option<T>
// where
//     T: TryFrom<ExtendedValue, Error = WalleError>,
// {
//     type Error = WalleError;
//     fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
//         match value {
//             ExtendedValue::Null => Ok(None),
//             v => Ok(Some(v.try_into()?)),
//         }
//     }
// }

/// json bytes could be String
impl TryFrom<Value> for OneBotBytes {
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bytes(v) => Ok(v),
            Value::Str(s) => Ok(OneBotBytes(
                base64::decode(&s).map_err(|_| WalleError::IllegalBase64(s))?,
            )),
            v => Err(WalleError::ValueTypeNotMatch(
                "bytes".to_string(),
                format!("{:?}", v),
            )),
        }
    }
}

impl TryFrom<Value> for () {
    type Error = WalleError;
    fn try_from(_: Value) -> Result<Self, Self::Error> {
        Ok(())
    }
}

macro_rules! downcast_fn {
    ($fname: ident, $ty: ty) => {
        pub fn $fname(self) -> WalleResult<$ty> {
            self.downcast()
        }
    };
}

#[allow(dead_code)]
impl Value {
    pub fn downcast<T>(self) -> WalleResult<T>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.try_into()
    }

    downcast_fn!(downcast_str, String);
    downcast_fn!(downcast_int, i64);
    downcast_fn!(downcast_f64, f64);
    downcast_fn!(downcast_bool, bool);
    downcast_fn!(downcast_bytes, OneBotBytes);

    pub fn downcast_map(self) -> WalleResult<HashMap<String, Value>> {
        if let Self::Map(m) = self {
            Ok(m)
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "ExtendedMap".to_string(),
                format!("{:?}", self),
            ))
        }
    }

    pub fn downcast_list(self) -> WalleResult<Vec<Value>> {
        if let Self::List(l) = self {
            Ok(l)
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "ExtendedList".to_string(),
                format!("{:?}", self),
            ))
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(&b.0),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&ValueMap> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_str_mut(&mut self) -> Option<&mut String> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            Self::F64(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            Self::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_bytes_mut(&mut self) -> Option<&mut [u8]> {
        match self {
            Self::Bytes(b) => Some(&mut b.0),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut ValueMap> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

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
    pub fn is_bytes(&self) -> bool {
        matches!(self, Self::Bytes(_))
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

pub trait PushToValueMap {
    fn push_to(self, map: &mut ValueMap);
}

impl PushToValueMap for () {
    fn push_to(self, _map: &mut ValueMap) {}
}

pub trait ValueMapExt {
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>;
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>;
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>;
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>;
    fn push<T>(&mut self, value: T)
    where
        T: PushToValueMap;
}

impl ValueMapExt for ValueMap {
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.remove(key)
            .and_then(|v| match v {
                Value::Null => None,
                v => Some(v.try_into().map_err(|v| {
                    let msg = format!("{:?}", v);
                    WalleError::ValueTypeNotMatch(std::any::type_name::<T>().to_string(), msg)
                })),
            })
            .transpose()
    }
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.try_remove_downcast(key)
            .and_then(|v| v.ok_or_else(|| WalleError::MapMissedKey(key.to_string())))
    }
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.get(key)
            .and_then(|v| match v {
                Value::Null => None,
                v => Some(v.clone().try_into().map_err(|v| {
                    let msg = format!("{:?}", v);
                    WalleError::ValueTypeNotMatch(std::any::type_name::<T>().to_string(), msg)
                })),
            })
            .transpose()
    }
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.try_get_downcast(key)
            .and_then(|v| v.ok_or_else(|| WalleError::MapMissedKey(key.to_string())))
    }
    fn push<T>(&mut self, value: T)
    where
        T: PushToValueMap,
    {
        value.push_to(self)
    }
}
