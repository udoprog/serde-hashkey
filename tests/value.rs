use serde_derive::{Deserialize, Serialize};
use serde_hashkey::{from_key, to_key, Error, Key};
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
    return Ok(());

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    enum Foo {
        Operation1(String, String),
        Operation2(String),
    }
}
