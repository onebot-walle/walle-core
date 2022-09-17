use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::collections::HashMap;

use super::OneBotBytes;
use crate::error::{WalleError, WalleResult};

/// 扩展字段 Map
pub type ValueMap = HashMap<String, Value>;

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

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("accepts string, f64, int, bool, map, list, null")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(Value::Int(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(Value::Int(v as i64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(Value::F64(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Str(v.to_owned()))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(OneBotBytes(v.to_owned())))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(OneBotBytes(v)))
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(Value::List(values))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::MapAccess<'de>,
    {
        let mut map = HashMap::new();
        while let Some((key, value)) = visitor.next_entry()? {
            map.insert(key, value);
        }
        Ok(Value::Map(map))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Null)
    }
}

impl<'de> Deserialize<'de> for Value {
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
    fn push_to(self, _: &mut ValueMap)
    where
        Self: Sized,
    {
    }
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

#[macro_export]
/// Value 声明宏，类似于`serde_json::json!`
macro_rules! value {
    (null) => {
        $crate::util::Value::Null
    };
    ([$($tt:tt)*]) => {
        $crate::util::Value::List($crate::value_vec![$($tt)*])
    };
    ({$($tt:tt)*}) => {
        $crate::util::Value::Map($crate::value_map!{$($tt)*})
    };
    ($s:expr) => {
        $s.to_owned().into()
    };
}

#[macro_export]
/// Vec<Value> 声明宏
macro_rules! value_vec {
    (@internal [$($elems:expr),*]) => {
        vec![$($elems),*]
    };
    (@internal [$($elems: expr,)*] null $($rest:tt)*) => {
        $crate::value_vec![@internal [$($elems,)* $crate::util::Value::Null] $($rest)*]
    };
    (@internal [$($elems: expr,)*] [$($vec: tt)*] $($rest:tt)*) => {
        $crate::value_vec![@internal [$($elems,)* $crate::value!([$($vec)*])] $($rest)*]
    };
    (@internal [$($elems: expr,)*] {$($map: tt)*} $($rest:tt)*) => {
        $crate::value_vec![@internal [$($elems,)* $crate::value!({$($map)*})] $($rest)*]
    };
    (@internal [$($elems: expr,)*] $t:expr, $($rest:tt)*) => {
        $crate::value_vec![@internal [$($elems,)* $crate::value!($t),] $($rest)*]
    };
    (@internal [$($elems: expr,)*] $t:expr) => {
        $crate::value_vec![@internal [$($elems,)* $crate::value!($t)]]
    };
    (@internal [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::value_vec![@internal [$($elems,)*] $($rest)*]
    };
    [$($tt: tt)*] => {
        $crate::value_vec!(@internal [] $($tt)*)
    };
}

#[macro_export]
/// ValueMap 声明宏
macro_rules! value_map {
    (@internal $map: ident {$key: expr} {$value: tt} ($($rest: tt)*)) => {
        let _ = $map.insert($key.into(), $crate::value!($value));
        $crate::value_map!(@internal $map () ($($rest)*));
    };
    (@internal $map: ident {$key: expr} {$value: tt}) => {
        let _ = $map.insert($key.into(), $crate::value!($value));
    };
    (@internal $map: ident {$key: expr} (: null $($rest:tt)*)) => {
        $crate::value_map!(@internal $map {$key} {null} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: [$($vec: tt)*] $($rest:tt)*)) => {
        $crate::value_map!(@internal $map {$key} {[$($vec)*]} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: {$($submap: tt)*} $($rest:tt)*)) => {
        $crate::value_map!(@internal $map {$key} {{$($submap)*}} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: $value: expr , $($rest:tt)*)) => {
        $crate::value_map!(@internal $map {$key} {$value} ($($rest)*));
    };
    (@internal $map: ident {$key: expr} (: $value: expr)) => {
        $crate::value_map!(@internal $map {$key} {$value});
    };
    (@internal $map: ident () ($key: tt: $($rest:tt)*)) => {
        $crate::value_map!(@internal $map {$key} (: $($rest)*));
    };
    (@internal $map: ident () (, $($rest: tt)*)) => {
        $crate::value_map!(@internal $map () ($($rest)*));
    };
    (@internal $map: ident () ()) => {};
    {$($tt:tt)*} => {
        {
            #[allow(unused_mut)]
            let mut map = $crate::util::ValueMap::default();
            $crate::value_map!(@internal map () ($($tt)*));
            map
        }
    };
}

#[test]
fn macro_test() {
    println!("{:?}", value!(null));
    println!(
        "{:?}",
        value_vec![true, 1, "c", 3., [1, 2, 3], {"a": 1, "b": 2}, Value::Bytes(vec![1, 2, 3].into())]
    );
    let a = "a";
    println!("{:?}", value!([1, "c", 3.]));
    println!(
        "{:?}",
        value_map! {
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

use serde::de::{Deserializer, SeqAccess};
use serde::forward_to_deserialize_any;

impl<'de> Deserializer<'de> for Value {
    type Error = WalleError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Int(i) => visitor.visit_i64(i),
            Value::F64(f) => visitor.visit_f64(f),
            Value::Str(s) => visitor.visit_string(s),
            Value::Bytes(b) => visitor.visit_byte_buf(b.0),
            Value::List(l) => visitor.visit_seq(SeqDezer(l.into_iter())),
            Value::Map(m) => visitor.visit_map(MapDezer {
                iter: m.into_iter(),
                value: None,
            }),
        }
    }
    forward_to_deserialize_any! {bool f32 f64 char str string bytes byte_buf
    unit unit_struct seq tuple tuple_struct map struct identifier ignored_any
    i8 i16 i32 i64 u8 u16 u32 u64 option enum newtype_struct}
}

struct SeqDezer(std::vec::IntoIter<Value>);

impl<'de> SeqAccess<'de> for SeqDezer {
    type Error = WalleError;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

struct MapDezer {
    iter: std::collections::hash_map::IntoIter<String, Value>,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for MapDezer {
    type Error = WalleError;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(StringDezer(key)).map(Some)
            }
            None => Ok(None),
        }
    }
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(WalleError::Other("map value missed".to_owned())),
        }
    }
}

struct StringDezer(String);

impl<'de> Deserializer<'de> for StringDezer {
    type Error = WalleError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }
    forward_to_deserialize_any! {bool f32 f64 char str string bytes byte_buf
    unit unit_struct seq tuple tuple_struct map struct identifier ignored_any
    i8 i16 i32 i64 u8 u16 u32 u64 option enum newtype_struct}
}

impl<'de> Deserializer<'de> for MapDezer {
    type Error = WalleError;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }
    forward_to_deserialize_any! {bool f32 f64 char str string bytes byte_buf
    unit unit_struct seq tuple tuple_struct map struct identifier ignored_any
    i8 i16 i32 i64 u8 u16 u32 u64 option enum newtype_struct}
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null => serializer.serialize_none(),
            Value::Str(s) => serializer.serialize_str(s),
            Value::F64(f) => serializer.serialize_f64(*f),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Int(i) => serializer.serialize_i64(*i),
            Value::Bytes(b) => b.serialize(serializer),
            Value::List(l) => l.serialize(serializer),
            Value::Map(m) => m.serialize(serializer),
        }
    }
}

pub fn from_value<T>(value: Value) -> Result<T, WalleError>
where
    T: for<'de> Deserialize<'de>,
{
    T::deserialize(value)
}

pub fn from_value_map<T>(map: ValueMap) -> Result<T, WalleError>
where
    T: for<'de> Deserialize<'de>,
{
    T::deserialize(MapDezer {
        iter: map.into_iter(),
        value: None,
    })
}

struct Serializer;

macro_rules! number_impl {
    ($fn_name: ident, $t: tt, $t0: ty) => {
        fn $fn_name(self, v: $t0) -> Result<Self::Ok, Self::Error> {
            Ok(Value::$t(v))
        }
    };
    ($fn_name: ident, $t: tt, $t0: ty, $t1: ty) => {
        fn $fn_name(self, v: $t0) -> Result<Self::Ok, Self::Error> {
            Ok(Value::$t(v as $t1))
        }
    };
}

impl serde::ser::Serializer for Serializer {
    type Ok = Value;
    type Error = WalleError;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeMapVariant;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeSeqVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }
    number_impl!(serialize_i8, Int, i8, i64);
    number_impl!(serialize_i16, Int, i16, i64);
    number_impl!(serialize_i32, Int, i32, i64);
    number_impl!(serialize_i64, Int, i64);
    number_impl!(serialize_i128, Int, i128, i64);
    number_impl!(serialize_u8, Int, u8, i64);
    number_impl!(serialize_u16, Int, u16, i64);
    number_impl!(serialize_u32, Int, u32, i64);
    number_impl!(serialize_u64, Int, u64, i64);
    number_impl!(serialize_u128, Int, u128, i64);
    number_impl!(serialize_f32, F64, f32, f64);
    number_impl!(serialize_f64, F64, f64);
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bytes(OneBotBytes(v.to_vec())))
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(v.to_string()))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: if let Some(len) = len {
                HashMap::with_capacity(len)
            } else {
                HashMap::new()
            },
            key: None,
        })
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(Value::Map(HashMap::from([(
            variant.to_owned(),
            value.serialize(self)?,
        )])))
    }
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Ok(value.serialize(self)?)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq(if let Some(len) = len {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        }))
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(v.to_owned()))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeMap {
            map: HashMap::with_capacity(len),
            key: None,
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeMapVariant {
            map: HashMap::with_capacity(len),
            name: variant.to_owned(),
        })
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len)))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeSeq(Vec::with_capacity(len)))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeSeqVariant {
            name: variant.to_owned(),
            seq: Vec::with_capacity(len),
        })
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null)
    }
    fn serialize_unit_variant(
        self,
        _sname: &'static str,
        _svariant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Str(variant.to_owned()))
    }
}

struct SerializeMap {
    map: HashMap<String, Value>,
    key: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.key = Some(serde_json::to_string(key).map_err(|e| WalleError::Other(e.to_string()))?);
        Ok(())
    }
    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        match self.key.take() {
            Some(key) => {
                self.map.insert(key, to_value(value)?);
                Ok(())
            }
            None => Err(WalleError::Other("miss key".to_owned())),
        }
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(self.map))
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.map.insert(key.to_owned(), to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(self.map))
    }
}

struct SerializeMapVariant {
    map: HashMap<String, Value>,
    name: String,
}

impl serde::ser::SerializeStructVariant for SerializeMapVariant {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.map.insert(key.to_owned(), to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(HashMap::from([(
            self.name,
            Value::Map(self.map),
        )])))
    }
}

struct SerializeSeq(Vec<Value>);

impl serde::ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.push(to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }
}

impl serde::ser::SerializeTuple for SerializeSeq {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.push(to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }
}

impl serde::ser::SerializeTupleStruct for SerializeSeq {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.0.push(to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.0))
    }
}

struct SerializeSeqVariant {
    name: String,
    seq: Vec<Value>,
}

impl serde::ser::SerializeTupleVariant for SerializeSeqVariant {
    type Ok = Value;
    type Error = WalleError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.seq.push(to_value(value)?);
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(HashMap::from([(
            self.name,
            Value::List(self.seq),
        )])))
    }
}

pub fn to_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, WalleError> {
    value.serialize(Serializer)
}
