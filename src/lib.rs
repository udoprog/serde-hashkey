//! In-memory key serialization.

mod de;
mod error;
mod key;
mod ser;

#[doc(inline)]
pub use crate::de::from_key;
#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::key::Key;
#[doc(inline)]
pub use crate::ser::to_key;
