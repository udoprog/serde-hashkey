#![cfg(feature = "ordered-float")]

use serde_derive::{Deserialize, Serialize};
use serde_hashkey::{
    from_key, to_key, to_key_with_ordered_float, Error, Float, Integer, Key, OrderedFloat,
};
use std::collections::BTreeMap;

#[test]
fn test_map() -> Result<(), Error> {
    let foo = Foo {
        name: String::from("Hello World"),
    };

    let mut map = BTreeMap::new();
    map.insert(&foo, String::from("bar"));

    match to_key(&map)? {
        Key::Map(_) => (),
        other => panic!("unexpected: {:?}", other),
    }

    match to_key(&foo)? {
        Key::Map(_) => (),
        other => panic!("unexpected: {:?}", other),
    }

    return Ok(());

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
    struct Foo {
        name: String,
    }
}

#[test]
fn test_enum() -> Result<(), Error> {
    let foo = Foo::Operation1(String::from("Foo"), String::from("Bar"));
    let value = to_key(&foo)?;

    match &value {
        Key::Map(_) => (),
        other => panic!("unexpected: {:?}", other),
    }

    assert_eq!(foo, from_key(&value)?);
    assert_eq!(Foo::Operation3, from_key(&to_key(&Foo::Operation3)?)?);
    return Ok(());

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    enum Foo {
        Operation1(String, String),
        Operation2(String),
        Operation3,
    }
}

#[test]
fn test_width() {
    assert_eq!(24, std::mem::size_of::<Integer>());
    assert!(std::mem::size_of::<Key>() <= 32 && std::mem::size_of::<Key>() >= 24);
}

#[test]
fn test_normalize() {
    let a = Key::Map(
        vec![
            (Key::String("baz".into()), Key::String("biz".into())),
            (Key::String("foo".into()), Key::String("bar".into())),
        ]
        .into(),
    );

    let b = Key::Map(
        vec![
            (Key::String("foo".into()), Key::String("bar".into())),
            (Key::String("baz".into()), Key::String("biz".into())),
        ]
        .into(),
    );

    assert_ne!(a, b);
    assert_eq!(a, b.clone().normalize());
    assert_eq!(a.clone().normalize(), b.clone().normalize());
}

#[test]
fn deny_floats_by_default() {
    assert_eq!(to_key(&0f32), Err(Error::UnsupportedType("f32")));
    assert_eq!(to_key(&0f64), Err(Error::UnsupportedType("f64")));
    assert_eq!(
        to_key_with_ordered_float(&0f32),
        Ok(Key::Float(Float::F32(OrderedFloat(0f32))))
    );
    assert_eq!(
        to_key_with_ordered_float(&0f64),
        Ok(Key::Float(Float::F64(OrderedFloat(0f64))))
    );
}
