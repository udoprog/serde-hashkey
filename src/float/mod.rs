#[cfg(feature = "ordered-float")]
pub use self::ordered_float::{to_key_with_ordered_float, OrderedFloat};
use crate::error::Error;
use serde::{de, ser};
use std::hash::Hash;

#[cfg(feature = "ordered-float")]
mod ordered_float;

// NB: we completely seal the FloatPolicy to prevent external implementations.
mod private {
    pub trait Sealed {}

    impl<T> Sealed for T where T: super::FloatPolicy {}
}

/// A policy for handling floating point types in a `Key`.
///
/// Currently there are two important `FloatPolicy` types: [`RejectFloat`] and
/// [`OrderedFloat`]. The former will emit errors instead of allowing floats to
/// be serialized and the latter while serialize them and provide a total order
/// which does not adhere to the IEEE standard.
///
/// [`RejectFloat`]: RejectFloat
/// [`OrderedFloat`]: OrderedFloat
pub trait FloatPolicy:
    self::private::Sealed + Clone + PartialEq + Eq + PartialOrd + Ord + Hash
{
    /// Serialize an `f32`, possibly failing.
    fn serialize_f32(value: f32) -> Result<Self, Error>;

    /// Serialize an `f64`, possibly failing.
    fn serialize_f64(value: f64) -> Result<Self, Error>;

    /// Serialize some floating point type, possibly failing.
    fn serialize_float<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer;

    /// Deserialize some other type from this floating point type, possibly failing.
    fn deserialize_float<'de, V>(&self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>;
}

/// A float serialization policy which rejects any attempt to serialize a float with an error.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RejectFloat {}

impl FloatPolicy for RejectFloat {
    fn serialize_f32(_: f32) -> Result<Self, Error> {
        Err(Error::UnsupportedType("f32"))
    }

    fn serialize_f64(_: f64) -> Result<Self, Error> {
        Err(Error::UnsupportedType("f64"))
    }

    fn serialize_float<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {}
    }

    fn deserialize_float<'de, V>(&self, _visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match *self {}
    }
}
