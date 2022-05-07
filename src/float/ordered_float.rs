use crate::error::Error;
use crate::float::{FloatPolicy, FloatRepr};
use crate::key::Key;
use num_traits02 as nt02;
use ordered_float3 as of3;
use serde::{de, ser};
use std::cmp;
use std::fmt;
use std::hash;

/// An opaque floating-point representation which has a total ordering. This is
/// used by [OrderedFloatPolicy].
#[derive(Clone, Copy)]
pub struct OrderedFloat<T>(pub T);

impl<T> fmt::Debug for OrderedFloat<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl FloatRepr<f32> for OrderedFloat<f32> {
    fn serialize(float: f32) -> Result<Self, Error> {
        Ok(OrderedFloat(float))
    }

    fn visit<'de, V>(&self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.0)
    }
}

impl FloatRepr<f64> for OrderedFloat<f64> {
    fn serialize(float: f64) -> Result<Self, Error> {
        Ok(OrderedFloat(float))
    }

    fn visit<'de, V>(&self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.0)
    }
}

impl<T> serde::Serialize for OrderedFloat<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<T> PartialEq for OrderedFloat<T>
where
    T: nt02::Float,
{
    fn eq(&self, other: &Self) -> bool {
        of3::OrderedFloat(self.0) == of3::OrderedFloat(other.0)
    }
}

impl<T> Eq for OrderedFloat<T> where T: nt02::Float {}

impl<T> PartialOrd for OrderedFloat<T>
where
    T: nt02::Float,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(of3::OrderedFloat(self.0).cmp(&of3::OrderedFloat(other.0)))
    }
}

impl<T> cmp::Ord for OrderedFloat<T>
where
    T: nt02::Float,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        of3::OrderedFloat(self.0).cmp(&of3::OrderedFloat(other.0))
    }
}

impl<T> hash::Hash for OrderedFloat<T>
where
    T: nt02::Float,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        of3::OrderedFloat(self.0).hash(state)
    }
}

/// A float serialization policy which delegates decisions to the
/// `ordered-float` crate. This policy is used by the
/// [to_key_with_ordered_float] function.
///
/// [Key]: crate::Key
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct OrderedFloatPolicy(());

impl FloatPolicy for OrderedFloatPolicy {
    type F32 = OrderedFloat<f32>;
    type F64 = OrderedFloat<f64>;
}

/// Serialize the given value to a [Key] using [OrderedFloatPolicy].
///
/// This policy is derived from the [`OrderedFloat` type] in the
/// [`ordered-float` crate].
///
/// [`OrderedFloat` type]: https://docs.rs/ordered-float/2/ordered_float/struct.OrderedFloat.html
/// [`ordered-float` crate]: https://docs.rs/ordered-float/2/ordered_float/
///
/// # Examples
///
/// ```rust
/// use serde_derive::{Deserialize, Serialize};
/// use serde_hashkey::{from_key, to_key_with_ordered_float, OrderedFloat, Key};
/// use std::collections::HashMap;
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Author {
///     name: String,
///     age: f32,
/// }
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Book {
///     title: String,
///     author: Author,
/// }
///
/// # fn main() -> serde_hashkey::Result<()> {
/// let book = Book {
///     title: String::from("Birds of a feather"),
///     author: Author {
///         name: String::from("Noah"),
///         age: 42.5,
///     },
/// };
///
/// let key = to_key_with_ordered_float(&book)?;
/// let book2 = from_key(&key)?;
///
/// assert_eq!(book, book2);
/// # Ok(())
/// # }
/// ```
pub fn to_key_with_ordered_float<T>(value: &T) -> Result<Key<OrderedFloatPolicy>, Error>
where
    T: ser::Serialize,
{
    crate::ser::to_key_with_policy::<T, OrderedFloatPolicy>(value)
}
