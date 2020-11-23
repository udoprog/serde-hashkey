use crate::float::FloatRepr;

/// A policy for handling floating point types in a `Key`.
///
/// Currently there are two important `FloatPolicy` types: [RejectFloatPolicy]
/// and [OrderedFloat]. The former will emit errors instead of allowing floats
/// to be serialized and the latter while serialize them and provide a total
/// order which does not adhere to the IEEE standard.
///
/// [RejectFloatPolicy]: crate::RejectFloatPolicy
/// [OrderedFloat]: crate::OrderedFloat
///
/// # Examples
///
/// Example using a non-default float policy:
///
/// ```rust
/// use serde_hashkey::{Key, Float, to_key_with_ordered_float, OrderedFloat, OrderedFloatPolicy};
///
/// # fn main() -> Result<(), serde_hashkey::Error> {
/// let a: Key<OrderedFloatPolicy> = to_key_with_ordered_float(&42.42f32)?;
/// assert!(matches!(a, Key::Float(Float::F32(OrderedFloat(..)))));
///
/// let b: Key<OrderedFloatPolicy> = to_key_with_ordered_float(&42.42f64)?;
/// assert!(matches!(b, Key::Float(Float::F64(OrderedFloat(..)))));
/// # Ok(()) }
/// ```
pub trait FloatPolicy: self::private::Sealed {
    /// The type encapsulating a 32-bit float, or `f32`.
    type F32: FloatRepr<f32>;

    /// The type encapsulating a 64-bit float, or `f64`.
    type F64: FloatRepr<f64>;
}

// NB: we completely seal the FloatPolicy to prevent external implementations.
mod private {
    pub trait Sealed {}
    impl<T> Sealed for T where T: super::FloatPolicy {}
}
