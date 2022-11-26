use std::collections::HashMap;

use super::{OneBotBytes, TryAsMut, TryAsRef};
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

    pub fn try_as_ref<T>(&self) -> WalleResult<T>
    where
        Self: for<'a> TryAsRef<'a, T>,
    {
        self._try_as_ref()
    }

    pub fn try_as_mut<T>(&mut self) -> WalleResult<T>
    where
        Self: for<'a> TryAsMut<'a, T>,
    {
        self._try_as_mut()
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
    fn try_get_as_ref<'a, T>(&'a self, key: &str) -> Result<T, WalleError>
    where
        Value: TryAsRef<'a, T>,
        T: 'a;
    fn try_get_as_mut<'a, T>(&'a mut self, key: &str) -> Result<T, WalleError>
    where
        Value: TryAsMut<'a, T>,
        T: 'a;
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
    fn try_get_as_ref<'a, T>(&'a self, key: &str) -> Result<T, WalleError>
    where
        Value: TryAsRef<'a, T>,
        T: 'a,
    {
        match self.get(key) {
            Some(v) => v._try_as_ref(),
            None => Err(WalleError::MapMissedKey(key.to_owned())),
        }
    }
    fn try_get_as_mut<'a, T>(&'a mut self, key: &str) -> Result<T, WalleError>
    where
        Value: TryAsMut<'a, T>,
        T: 'a,
    {
        match self.get_mut(key) {
            Some(v) => v._try_as_mut(),
            None => Err(WalleError::MapMissedKey(key.to_owned())),
        }
    }
    fn push<T>(&mut self, value: T)
    where
        T: PushToValueMap,
    {
        value.push_to(self)
    }
}

macro_rules! ref_impl {
    ($t: ty, $mt: ty, $subt: tt, $s: expr) => {
        impl<'a> TryAsRef<'a, $t> for Value {
            fn _try_as_ref(&'a self) -> WalleResult<$t> {
                match self {
                    Self::$subt(r) => Ok(r),
                    _ => Err(WalleError::ValueTypeNotMatch(
                        $s.to_owned(),
                        format!("{:?}", self),
                    )),
                }
            }
        }
        impl<'a> TryAsMut<'a, $mt> for Value {
            fn _try_as_mut(&'a mut self) -> WalleResult<$mt> {
                match self {
                    Self::$subt(r) => Ok(r),
                    _ => Err(WalleError::ValueTypeNotMatch(
                        $s.to_owned(),
                        format!("{:?}", self),
                    )),
                }
            }
        }
    };
}

impl<'a> TryAsRef<'a, &'a str> for Value {
    fn _try_as_ref(&'a self) -> WalleResult<&'a str> {
        match self {
            Self::Str(s) => Ok(s),
            _ => Err(WalleError::ValueTypeNotMatch(
                "str".to_string(),
                format!("{:?}", self),
            )),
        }
    }
}

impl<'a> TryAsMut<'a, &'a mut String> for Value {
    fn _try_as_mut(&'a mut self) -> WalleResult<&'a mut String> {
        match self {
            Self::Str(s) => Ok(s),
            _ => Err(WalleError::ValueTypeNotMatch(
                "str".to_string(),
                format!("{:?}", self),
            )),
        }
    }
}

ref_impl!(&'a i64, &'a mut i64, Int, "i64");
ref_impl!(&'a f64, &'a mut f64, F64, "f64");
ref_impl!(&'a bool, &'a mut bool, Bool, "bool");
ref_impl!(&'a ValueMap, &'a mut ValueMap, Map, "map");
ref_impl!(&'a Vec<Value>, &'a mut Vec<Value>, List, "list");
ref_impl!(&'a OneBotBytes, &'a mut OneBotBytes, Bytes, "bytes");
