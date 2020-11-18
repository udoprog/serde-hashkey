use serde_derive::{Deserialize, Serialize};
use serde_hashkey::{from_key, to_key, Error, Integer, Key};
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

    assert_eq!(foo, from_key::<Foo>(&value)?);
    assert_eq!(
        Foo::Operation3,
        from_key::<Foo>(&to_key(&Foo::Operation3)?)?
    );
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
    assert_eq!(32, std::mem::size_of::<Key>());
}

#[test]
fn test_normalize() {
    let a = Key::Map(vec![
        (
            Key::String(String::from("baz")),
            Key::String(String::from("biz")),
        ),
        (
            Key::String(String::from("foo")),
            Key::String(String::from("bar")),
        ),
    ]);

    let b = Key::Map(vec![
        (
            Key::String(String::from("foo")),
            Key::String(String::from("bar")),
        ),
        (
            Key::String(String::from("baz")),
            Key::String(String::from("biz")),
        ),
    ]);

    assert_ne!(a, b);
    assert_eq!(a, b.clone().normalize());
    assert_eq!(a.clone().normalize(), b.clone().normalize());
}
