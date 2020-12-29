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
//! Serde-based in-memory key serialization which supports hashing.
//!
//! This allows any serde-serializable type to be converted into a value which
//! implements `PartialEq`, `Eq`, `ParialOrd`, `Ord`, and `Hash`.
//!
//! [Key] is useful because it allows for a form of type-erasure. Let's say you
//! want to build a generic in-memory key-value store where you want to store
//! arbitrary serde-serializable keys. This is typical for things like caches or
//! dependency injection frameworks.
//!
//! ## Float policies
//!
//! By default, [Key] can't include floating point types such as `f32` and
//! `f64`. Neither of these are [totally ordered nor hashable].
//!
//! To enable the [Key] type to use `f32` and `f64` it can be constructed with a
//! specific float policy.
//!
//! Available float policies are:
//! * [RejectFloat] - the default behavior when using [to_key].
//! * [OrderedFloat] - the behavior when using [to_key_with_ordered_float]. The
//!   `ordered-float` feature must be enabled to use this. The behavior is
//!   derived from the [`ordered-float` crate].
//!
//! ## Features
//!
//! * `ordered-float` - Enables serializing floating point numbers through
//!   behavior derived from the [`ordered-float` crate]
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
//! [to_key]: https://docs.rs/serde-hashkey/0/serde_hashkey/fn.to_key.html
//! [RejectFloat]: https://docs.rs/serde-hashkey/0/serde_hashkey/enum.RejectFloat.html
//! [OrderedFloat]: https://docs.rs/serde-hashkey/0/serde_hashkey/enum.OrderedFloat.html
//! [to_key_with_ordered_float]: https://docs.rs/serde-hashkey/0/serde_hashkey/fn.to_key_with_ordered_float.html
//! [`ordered-float` crate]: https://docs.rs/ordered-float/2/ordered_float/

#![deny(missing_docs)]

mod de;
mod error;
mod float;
mod key;
mod ser;

#[doc(inline)]
pub use crate::de::from_key;
#[doc(inline)]
pub use crate::error::{Error, Result};
#[cfg(feature = "ordered-float")]
pub use crate::float::{to_key_with_ordered_float, OrderedFloat, OrderedFloatPolicy};
pub use crate::float::{FloatPolicy, FloatRepr, NeverFloat, RejectFloatPolicy};
#[doc(inline)]
pub use crate::key::{Float, Integer, Key};
#[doc(inline)]
pub use crate::ser::to_key;
