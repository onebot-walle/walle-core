use serde::{de::Visitor, Deserialize, Serialize};
use std::collections::HashMap;

use super::OneBotBytes;
use crate::error::{WalleError, WalleResult};

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
    Map(ExtendedMap),
    List(Vec<ExtendedValue>),
    Bytes(OneBotBytes),
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
// impl_from!(Int, u8, i64);
impl_from!(Int, u16, i64);
impl_from!(Int, u32, i64);
impl_from!(F64, f64);
impl_from!(F64, f32, f64);
impl_from!(Bool, bool);
impl_from!(Bytes, OneBotBytes);

impl From<()> for ExtendedValue {
    fn from(_: ()) -> Self {
        ExtendedValue::Null
    }
}

impl From<Vec<u8>> for ExtendedValue {
    fn from(v: Vec<u8>) -> Self {
        ExtendedValue::Bytes(OneBotBytes(v))
    }
}

impl From<&[u8]> for ExtendedValue {
    fn from(v: &[u8]) -> Self {
        ExtendedValue::Bytes(OneBotBytes(v.to_vec()))
    }
}

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

impl<T> From<Option<T>> for ExtendedValue
where
    T: Into<ExtendedValue>,
{
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => ExtendedValue::Null,
        }
    }
}

macro_rules! impl_try_from {
    ($inty: tt, $ty: ty) => {
        impl TryFrom<ExtendedValue> for $ty {
            type Error = WalleError;
            fn try_from(i: ExtendedValue) -> Result<$ty, Self::Error> {
                match i {
                    ExtendedValue::$inty(v) => Ok(v),
                    v => Err(WalleError::ValueTypeNotMatch(
                        std::any::type_name::<$ty>().to_string(),
                        format!("{:?}", v),
                    )),
                }
            }
        }
    };
    ($inty: tt as $ty: ty) => {
        impl TryFrom<ExtendedValue> for $ty {
            type Error = WalleError;
            fn try_from(i: ExtendedValue) -> Result<$ty, Self::Error> {
                match i {
                    ExtendedValue::$inty(v) => Ok(v as $ty),
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

impl<V> TryFrom<ExtendedValue> for Vec<V>
where
    V: TryFrom<ExtendedValue, Error = WalleError>,
{
    type Error = WalleError;
    fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
        match value {
            ExtendedValue::List(l) => l.into_iter().map(|v| v.try_into()).collect(),
            v => Err(WalleError::ValueTypeNotMatch(
                format!("List<{}>", std::any::type_name::<V>()),
                format!("{:?}", v),
            )),
        }
    }
}

impl<V> TryFrom<ExtendedValue> for HashMap<String, V>
where
    V: TryFrom<ExtendedValue, Error = WalleError>,
{
    type Error = WalleError;
    fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
        match value {
            ExtendedValue::Map(m) => m
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
impl TryFrom<ExtendedValue> for OneBotBytes {
    type Error = WalleError;
    fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
        match value {
            ExtendedValue::Bytes(v) => Ok(v),
            ExtendedValue::Str(s) => Ok(OneBotBytes(
                base64::decode(&s).map_err(|_| WalleError::IllegalBase64(s))?,
            )),
            v => Err(WalleError::ValueTypeNotMatch(
                "bytes".to_string(),
                format!("{:?}", v),
            )),
        }
    }
}

fn null_serialize<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_none()
}

// fn null_deserialize<'de, D>(deserializer: D) -> Result<ExtendedValue, D::Error>
// where
//     D: serde::de::Deserializer<'de>,
// {
//     deserializer.deserialize_unit(ValueVisitor)
// }

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

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtendedValue::Bytes(OneBotBytes(v.to_owned())))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ExtendedValue::Bytes(OneBotBytes(v)))
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
        pub fn $fname(self) -> WalleResult<$ty> {
            self.downcast()
        }
    };
}

#[allow(dead_code)]
impl ExtendedValue {
    pub fn downcast<T>(self) -> WalleResult<T>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>,
    {
        self.try_into()
    }

    downcast_fn!(downcast_str, String);
    downcast_fn!(downcast_int, i64);
    downcast_fn!(downcast_f64, f64);
    downcast_fn!(downcast_bool, bool);
    downcast_fn!(downcast_bytes, OneBotBytes);

    pub fn downcast_map(self) -> WalleResult<HashMap<String, ExtendedValue>> {
        if let Self::Map(m) = self {
            Ok(m)
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "ExtendedMap".to_string(),
                format!("{:?}", self),
            ))
        }
    }

    pub fn downcast_list(self) -> WalleResult<Vec<ExtendedValue>> {
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

    pub fn as_map(&self) -> Option<&ExtendedMap> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ExtendedValue>> {
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

pub trait PushToExtendedMap {
    fn push_to(self, _: &mut ExtendedMap)
    where
        Self: Sized,
    {
    }
}

pub trait ExtendedMapExt {
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>;
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>;
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>;
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>;
    fn push<T>(&mut self, value: T)
    where
        T: PushToExtendedMap;
}

impl ExtendedMapExt for ExtendedMap {
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>,
    {
        self.remove(key)
            .map(|v| {
                v.try_into().map_err(|v| {
                    let msg = format!("{:?}", v);
                    WalleError::ValueTypeNotMatch(std::any::type_name::<T>().to_string(), msg)
                })
            })
            .transpose()
    }
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>,
    {
        self.try_remove_downcast(key)
            .and_then(|v| v.ok_or_else(|| WalleError::MapMissedKey(key.to_string())))
    }
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>,
    {
        self.get(key)
            .map(|v| {
                v.clone().try_into().map_err(|v| {
                    let msg = format!("{:?}", v);
                    WalleError::ValueTypeNotMatch(std::any::type_name::<T>().to_string(), msg)
                })
            })
            .transpose()
    }
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<ExtendedValue, Error = WalleError>,
    {
        self.try_get_downcast(key)
            .and_then(|v| v.ok_or_else(|| WalleError::MapMissedKey(key.to_string())))
    }
    fn push<T>(&mut self, value: T)
    where
        T: PushToExtendedMap,
    {
        value.push_to(self)
    }
}

#[macro_export]
/// ExtendedValue 声明宏，类似于`serde_json::json!`
macro_rules! extended_value {
    (null) => {
        $crate::util::ExtendedValue::Null
    };
    ([$($tt:tt)*]) => {
        $crate::util::ExtendedValue::List($crate::extended_vec![$($tt)*])
    };
    ({$($tt:tt)*}) => {
        $crate::util::ExtendedValue::Map($crate::extended_map!{$($tt)*})
    };
    ($s:expr) => {
        $s.to_owned().into()
    };
}

#[macro_export]
/// Vec<ExtendedValue> 声明宏
macro_rules! extended_vec {
    (@internal [$($elems:expr),*]) => {
        vec![$($elems),*]
    };
    (@internal [$($elems: expr,)*] null $($rest:tt)*) => {
        $crate::extended_vec![@internal [$($elems,)* $crate::util::ExtendedValue::Null] $($rest)*]
    };
    (@internal [$($elems: expr,)*] [$($vec: tt)*] $($rest:tt)*) => {
        $crate::extended_vec![@internal [$($elems,)* $crate::extended_value!([$($vec)*])] $($rest)*]
    };
    (@internal [$($elems: expr,)*] {$($map: tt)*} $($rest:tt)*) => {
        $crate::extended_vec![@internal [$($elems,)* $crate::extended_value!({$($map)*})] $($rest)*]
    };
    (@internal [$($elems: expr,)*] $t:expr, $($rest:tt)*) => {
        $crate::extended_vec![@internal [$($elems,)* $crate::extended_value!($t),] $($rest)*]
    };
    (@internal [$($elems: expr,)*] $t:expr) => {
        $crate::extended_vec![@internal [$($elems,)* $crate::extended_value!($t)]]
    };
    (@internal [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::extended_vec![@internal [$($elems,)*] $($rest)*]
    };
    [$($tt: tt)*] => {
        $crate::extended_vec!(@internal [] $($tt)*)
    };
}

#[macro_export]
/// ExtendedMap 声明宏
macro_rules! extended_map {
    (@internal $map: ident {$key: expr} {$value: tt} ($($rest: tt)*)) => {
        let _ = $map.insert($key.into(), $crate::extended_value!($value));
        $crate::extended_map!(@internal $map () ($($rest)*));
    };
    (@internal $map: ident {$key: expr} {$value: tt}) => {
        let _ = $map.insert($key.into(), $crate::extended_value!($value));
    };
    (@internal $map: ident {$key: expr} (: null $($rest:tt)*)) => {
        $crate::extended_map!(@internal $map {$key} {null} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: [$($vec: tt)*] $($rest:tt)*)) => {
        $crate::extended_map!(@internal $map {$key} {[$($vec)*]} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: {$($submap: tt)*} $($rest:tt)*)) => {
        $crate::extended_map!(@internal $map {$key} {{$($submap)*}} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: $value: expr , $($rest:tt)*)) => {
        $crate::extended_map!(@internal $map {$key} {$value} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: $value: expr)) => {
        $crate::extended_map!(@internal $map {$key} {$value});
    };
    (@internal $map: ident () ($key: tt: $($rest:tt)*)) => {
        $crate::extended_map!(@internal $map {$key} (: $($rest)*));
    };
    (@internal $map: ident () (, $($rest: tt)*)) => {
        $crate::extended_map!(@internal $map () ($($rest)*));
    };
    (@internal $map: ident () ()) => {};
    {$($tt:tt)*} => {
        {
            #[allow(unused_mut)]
            let mut map = $crate::util::ExtendedMap::default();
            $crate::extended_map!(@internal map () ($($tt)*));
            map
        }
    };
}

#[test]
fn macro_test() {
    println!("{:?}", extended_value!(null));
    println!(
        "{:?}",
        extended_vec![true, 1, "c", 3., [1, 2, 3], {"a": 1, "b": 2}, ExtendedValue::Bytes(vec![1, 2, 3].into())]
    );
    let a = "a";
    println!("{:?}", extended_value!([1, "c", 3.]));
    println!(
        "{:?}",
        extended_map! {
            "a": a,
            "b": 2,
            "c": {
                "d": 3,
                "e": b"a"[..],
                "f": null
            }
        }
    );
}
