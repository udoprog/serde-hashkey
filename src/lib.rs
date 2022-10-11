//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/serde--hashkey-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/serde-hashkey)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/serde-hashkey.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/serde-hashkey)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-serde--hashkey-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/serde-hashkey)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/serde-hashkey/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/serde-hashkey/actions?query=branch%3Amain)
//!
//! Serde-based in-memory key serialization which supports hashing.
//!
//! This allows any serde-serializable type to be converted into a value which
//! implements `PartialEq`, `Eq`, `ParialOrd`, `Ord`, and `Hash`.
//!
//! [Key] is useful because it allows for a form of type-erasure. Let's say you
//! want to build a generic in-memory key-value store where you want to store
//! arbitrary serde-serializable keys. This is useful for things like caches or
//! dependency injection frameworks.
//!
//! <br>
//!
//! ## Usage
//!
//! Add the following to your Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! serde-hashkey = "0.4.3"
//! ```
//!
//! <br>
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
//! <br>
//!
//! ## Features
//!
//! * `ordered-float` - Enables serializing floating point numbers through
//!   behavior derived from the [`ordered-float` crate]
//!
//! <br>
//!
//! ## Examples
//!
//! > You can run this example with `cargo run --example book`
//!
//! ```
//! use std::collections::HashMap;
//!
//! use serde_derive::{Deserialize, Serialize};
//! use serde_hashkey::{from_key, to_key, Error, Key};
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
//! let book = Book {
//!     title: String::from("Birds of a feather"),
//!     author: Author {
//!         name: String::from("Noah"),
//!         age: 42,
//!     },
//! };
//!
//! let key = to_key(&book)?;
//!
//! let mut ratings = HashMap::new();
//! ratings.insert(key.clone(), 5);
//!
//! println!("ratings: {:?}", ratings);
//!
//! println!(
//!     "book as json (through key): {}",
//!     serde_json::to_string_pretty(&key)?
//! );
//!
//! println!(
//!     "book as json (through original object): {}",
//!     serde_json::to_string_pretty(&book)?
//! );
//!
//! # Ok::<_, Box<dyn std::error::Error>>(())
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
#![cfg_attr(docsrs, feature(doc_cfg))]

macro_rules! cfg_ordered_float {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "ordered-float")]
            #[cfg_attr(docsrs, doc(cfg(feature = "ordered-float")))]
            $item
        )*
    }
}

mod de;
mod error;
mod float;
mod key;
mod ser;

#[doc(inline)]
pub use crate::de::from_key;
#[doc(inline)]
pub use crate::error::{Error, Result};

cfg_ordered_float! {
    pub use crate::float::{to_key_with_ordered_float, OrderedFloat, OrderedFloatPolicy};
    pub use crate::float::{FloatPolicy, FloatRepr, NeverFloat, RejectFloatPolicy};
}

#[doc(inline)]
pub use crate::key::{Float, Integer, Key};
#[doc(inline)]
pub use crate::ser::to_key;
