//! <div align="center">
//!   <a href="https://github.com/udoprog/serde-hashkey/actions">
//!     <img alt="GitHub Actions Build Status" src="https://github.com/udoprog/serde-hashkey/workflows/Build/badge.svg">
//!   </a>
//!
//!   <a href="https://docs.rs/serde-hashkey">
//!     <img alt="Documentation" src="https://docs.rs/serde-hashkey/badge.svg">
//!   </a>
//! </div>
//!
//! Serde-based in-memory key serialization.
//!
//! This allows any serde-serializable type to be converted into a value which
//! implements `PartialEq`, `Eq`, `ParialOrd`, `Ord`, and `Hash`. The only
//! limitation is that the type can't serialize floating point values, since
//! these are not [totally ordered nor hashable] by default.
//!
//! [Key] is useful because it allows for a form of type-erasure. Let's say you
//! want to build a generic in-memory key-value store where you want to store
//! arbitrary serde-serializable keys. This is typical for things like caches or
//! dependency injection frameworks.
//!
//! ## Examples
//!
//! > You can run this example with `cargo run --example book`
//!
//! ```rust
//! use serde_derive::{Deserialize, Serialize};
//! use serde_hashkey::{from_key, to_key, Error, Key};
//! use std::{collections::HashMap, error};
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct Author {
//!     name: String,
//!     age: u32,
//! }
//!
//! #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
//! struct Book {
//!     title: String,
//!     author: Author,
//! }
//!
//! fn main() -> Result<(), Box<dyn error::Error>> {
//!     let book = Book {
//!         title: String::from("Birds of a feather"),
//!         author: Author {
//!             name: String::from("Noah"),
//!             age: 42,
//!         },
//!     };
//!
//!     let key = to_key(&book)?;
//!
//!     let mut ratings = HashMap::new();
//!     ratings.insert(key.clone(), 5);
//!
//!     println!("ratings: {:?}", ratings);
//!
//!     println!(
//!         "book as json (through key): {}",
//!         serde_json::to_string_pretty(&key)?
//!     );
//!
//!     println!(
//!         "book as json (through original object): {}",
//!         serde_json::to_string_pretty(&book)?
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! [totally ordered nor hashable]: https://internals.rust-lang.org/t/f32-f64-should-implement-hash/5436
//! [Key]: https://docs.rs/serde-hashkey/0/serde_hashkey/enum.Key.html

#![deny(missing_docs)]

mod de;
mod error;
mod key;
mod ser;

#[doc(inline)]
pub use crate::de::from_key;
#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::key::{Integer, Key};
#[doc(inline)]
pub use crate::ser::to_key;
