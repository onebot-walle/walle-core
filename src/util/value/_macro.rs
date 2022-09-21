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
        value_vec![true, 1, "c", 3., [1, 2, 3], {"a": 1, "b": 2}, crate::util::Value::Bytes(vec![1, 2, 3].into())]
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
