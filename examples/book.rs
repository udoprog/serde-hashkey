use serde_derive::{Deserialize, Serialize};
use serde_hashkey::to_key;
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
