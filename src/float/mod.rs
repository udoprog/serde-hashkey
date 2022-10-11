cfg_ordered_float! {
    pub use self::ordered_float::{to_key_with_ordered_float, OrderedFloat, OrderedFloatPolicy};
}

use crate::error::Error;
use serde::de;

mod float_policy;
mod float_repr;

cfg_ordered_float! {
    mod ordered_float;
}

pub use self::float_policy::FloatPolicy;
pub use self::float_repr::FloatRepr;

/// An uninhabitable type for float policies that cannot produce a value of the
/// corresponding type. This is used by [RejectFloatPolicy].
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum NeverFloat {}

impl FloatRepr<f32> for NeverFloat {
    fn serialize(_: f32) -> Result<Self, Error> {
        Err(Error::UnsupportedType("f32"))
    }

    fn visit<'de, V>(&self, _: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("f32"))
    }
}

impl FloatRepr<f64> for NeverFloat {
    fn serialize(_: f64) -> Result<Self, Error> {
        Err(Error::UnsupportedType("f64"))
    }

    fn visit<'de, V>(&self, _: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::UnsupportedType("f64"))
    }
}

impl serde::Serialize for NeverFloat {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Note: type is uninhabitable, so this impl can never be reached.
        unreachable!()
    }
}

/// A float serialization policy which rejects any attempt to serialize a float
/// with an error. This policy is used by the [to_key] function.
///
/// [to_key]: crate::to_key
///
/// # Examples
///
/// ```
/// use serde_hashkey::{Key, Float, to_key};
///
/// # fn main() -> Result<(), serde_hashkey::Error> {
/// assert!(to_key(&"John Doe").is_ok());
/// assert!(to_key(&42.42f32).is_err());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct RejectFloatPolicy(());

impl FloatPolicy for RejectFloatPolicy {
    type F32 = NeverFloat;
    type F64 = NeverFloat;
}
