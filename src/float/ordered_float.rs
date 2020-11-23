use crate::error::Error;
use crate::float::FloatPolicy;
use serde::{de, ser};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// An opaque floating-point type which has a total ordering.
#[derive(Debug, Clone, Copy)]
pub enum OrderedFloat {
    /// Variant for an `f32`.
    F32(f32),
    /// Variant for an `f64`.
    F64(f64),
}

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        use ordered_float::OrderedFloat as TotalOrd;
        match (*self, *other) {
            (Self::F32(lhs), Self::F32(rhs)) => TotalOrd(lhs) == TotalOrd(rhs),
            (Self::F64(lhs), Self::F64(rhs)) => TotalOrd(lhs) == TotalOrd(rhs),
            _ => false,
        }
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        use ordered_float::OrderedFloat as TotalOrd;
        match (*self, *other) {
            (Self::F32(_), Self::F64(_)) => Ordering::Less,
            (Self::F64(_), Self::F32(_)) => Ordering::Greater,
            (Self::F32(lhs), Self::F32(rhs)) => TotalOrd(lhs).cmp(&TotalOrd(rhs)),
            (Self::F64(lhs), Self::F64(rhs)) => TotalOrd(lhs).cmp(&TotalOrd(rhs)),
        }
    }
}

impl Hash for OrderedFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use ordered_float::OrderedFloat as TotalOrd;
        match *self {
            Self::F32(v) => TotalOrd(v).hash(state),
            Self::F64(v) => TotalOrd(v).hash(state),
        }
    }
}

impl FloatPolicy for OrderedFloat {
    fn serialize_f32(value: f32) -> Result<Self, Error> {
        Ok(Self::F32(value))
    }

    fn serialize_f64(value: f64) -> Result<Self, Error> {
        Ok(Self::F64(value))
    }

    fn serialize_float<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Self::F32(v) => serializer.serialize_f32(v),
            Self::F64(v) => serializer.serialize_f64(v),
        }
    }

    fn deserialize_float<'de, V>(&self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match *self {
            Self::F32(v) => visitor.visit_f32(v),
            Self::F64(v) => visitor.visit_f64(v),
        }
    }
}

/// Serialize the given value to a [Key], with an [OrderedFloat] float policy.
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
pub fn to_key_with_ordered_float<T>(value: &T) -> Result<crate::Key<OrderedFloat>, Error>
where
    T: ser::Serialize,
{
    crate::ser::to_key_with_policy::<T, OrderedFloat>(value)
}
