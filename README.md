# Serde Hash-Key Serialization

[![Build Status](https://travis-ci.org/udoprog/serde-hashkey.svg?branch=master)](https://travis-ci.org/udoprog/serde-hashkey)
[![Documentation](https://docs.rs/serde-hashkey/badge.svg)](https://docs.rs/serde-hashkey)

Serde-based in-memory key serialization.

This allows any serde-serializable type to be converted into a `Value` which implements `PartialEq`, `Eq`, `ParialOrd`, `Ord`, and `Hash`. The only limitation is that the type can't serialize floating point-types. This might be lifted in the future by specifying policies for dealing with non-finite values.

`Key` is useful because it allows for a form of type-erasure. Let's say you want to build a generic in-memory key-value store where you want to store arbitrary serde-serializable keys. This is typical for things like caches or dependency injection frameworks.

## Examples

Note: available as `cargo run --example book`

```rust
use serde_derive::{Deserialize, Serialize};
use serde_hashkey::{from_key, to_key, Error, Key};
use std::{collections::HashMap, error};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Author {
    name: String,
    age: u32,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Book {
    title: String,
    author: Author,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let book = Book {
        title: String::from("Birds of a feather"),
        author: Author {
            name: String::from("Noah"),
            age: 42,
        },
    };

    let key = to_key(&book)?;

    let mut ratings = HashMap::new();
    ratings.insert(key.clone(), 5);

    println!("ratings: {:?}", ratings);

    println!(
        "book as json (through key): {}",
        serde_json::to_string_pretty(&key)?
    );

    println!(
        "book as json (through original object): {}",
        serde_json::to_string_pretty(&book)?
    );

    Ok(())
}
```