use crate::error::Error;
use serde::{de, ser};
use std::cmp;
use std::fmt;
use std::hash;

/// Trait implemented by floating point types which can be used in a
/// [FloatPolicy]. This is implemented by the type representing a float,
/// typically a wrapper, and defines the protocol necessary to incorporate the
/// floating point type `T` into the [Key] protocol.
///
/// [Key]: crate::Key
/// [FloatPolicy]: crate::FloatPolicy
pub trait FloatRepr<T>:
    self::private::Sealed<T>
    + Copy
    + Sized
    + fmt::Debug
    + ser::Serialize
    + cmp::Eq
    + cmp::Ord
    + hash::Hash
{
    /// Serialize impl for a floating point value.
    fn serialize(value: T) -> Result<Self, Error>;

    /// Visit the current value.
    fn visit<'de, V>(&self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>;
}

// NB: we completely seal the FloatPolicy to prevent external implementations.
mod private {
    pub trait Sealed<A> {}
    impl<T, A> Sealed<A> for T where T: super::FloatRepr<A> {}
}
