//! In-memory value representation for values.
use crate::error::Error;
use serde_derive::*;
use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::hash::{Hash, Hasher};
use std::mem;

/// An opaque integer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Integer {
    /// Variant representing a `i8` integer.
    I8(i8),
    /// Variant representing a `i16` integer.
    I16(i16),
    /// Variant representing a `i32` integer.
    I32(i32),
    /// Variant representing a `i64` integer.
    I64(i64),
    /// Variant representing a `i128` integer.
    I128(i128),
    /// Variant representing a `u8` integer.
    U8(u8),
    /// Variant representing a `u16` integer.
    U16(u16),
    /// Variant representing a `u32` integer.
    U32(u32),
    /// Variant representing a `u64` integer.
    U64(u64),
    /// Variant representing a `u128` integer.
    U128(u128),
}

/// An opaque floating-point type which has a total ordering.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
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

impl TryFrom<f32> for OrderedFloat {
    type Error = Error;

    fn try_from(v: f32) -> Result<Self, Self::Error> {
        Ok(Self::F32(v))
    }
}

impl TryFrom<f64> for OrderedFloat {
    type Error = Error;

    fn try_from(v: f64) -> Result<Self, Self::Error> {
        Ok(Self::F64(v))
    }
}

impl From<OrderedFloat> for FloatProxy {
    fn from(ordered: OrderedFloat) -> Self {
        match ordered {
            OrderedFloat::F32(v) => FloatProxy::F32(v),
            OrderedFloat::F64(v) => FloatProxy::F64(v),
        }
    }
}

/// A float serialization policy which rejects any attempt to serialize a float with an error.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum RejectFloat {}

impl TryFrom<f32> for RejectFloat {
    type Error = Error;
    fn try_from(_: f32) -> Result<Self, Self::Error> {
        Err(Error::UnsupportedType("f32"))
    }
}

impl TryFrom<f64> for RejectFloat {
    type Error = Error;
    fn try_from(_: f64) -> Result<Self, Self::Error> {
        Err(Error::UnsupportedType("f64"))
    }
}

impl From<RejectFloat> for FloatProxy {
    fn from(this: RejectFloat) -> Self {
        match this {}
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FloatProxy {
    F32(f32),
    F64(f64),
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
    TryFrom<f32, Error = Error>
    + TryFrom<f64, Error = Error>
    + Into<FloatProxy>
    + Clone
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
{
}

impl<T> FloatPolicy for T where
    T: TryFrom<f32, Error = Error>
        + TryFrom<f64, Error = Error>
        + Into<FloatProxy>
        + Clone
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Hash
{
}

/// The central key type, which is an in-memory representation of all supported
/// serde-serialized values.
///
/// This can be serialized to a type implementing [serde::Deserialize] using
/// [from_key], and deserialized from a type implementing [serde::Serialize]
/// using [to_key]. See the corresponding function for documentation.
///
/// [from_key]: crate::from_key
/// [to_key]: crate::to_key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key<Float: FloatPolicy = RejectFloat> {
    /// A unit value.
    Unit,
    /// A boolean value.
    Bool(bool),
    /// An integer.
    Integer(Integer),
    /// A floating-point number.
    Float(Float),
    /// A byte array.
    Bytes(Vec<u8>),
    /// A string.
    String(String),
    /// A vector.
    Vec(Vec<Key<Float>>),
    /// A map.
    Map(Vec<(Key<Float>, Key<Float>)>),
}

impl Default for Key {
    fn default() -> Self {
        Self::Unit
    }
}

impl Key {
    /// Normalize the key, making sure that all contained maps are sorted.
    pub fn normalize(self) -> Key {
        match self {
            Key::Vec(mut vec) => {
                for value in &mut vec {
                    *value = mem::replace(value, Key::Unit).normalize();
                }

                Key::Vec(vec)
            }
            Key::Map(mut map) => {
                for (key, value) in &mut map {
                    *key = mem::replace(key, Key::Unit).normalize();
                    *value = mem::replace(value, Key::Unit).normalize();
                }

                map.sort_by(|a, b| a.0.cmp(&b.0));
                Key::Map(map)
            }
            other => other,
        }
    }
}

macro_rules! impl_integer_from {
    ($variant:ident, $for_type:ty) => {
        impl<Float: FloatPolicy> From<$for_type> for Key<Float> {
            fn from(v: $for_type) -> Key<Float> {
                Key::Integer(Integer::$variant(v))
            }
        }
    };
}

macro_rules! impl_float_try_from {
    ($variant:ident, $for_type:ty) => {
        impl<Float: FloatPolicy> TryFrom<$for_type> for Key<Float> {
            type Error = Error;

            fn try_from(v: $for_type) -> Result<Key<Float>, Error> {
                v.try_into().map(Key::Float)
            }
        }
    };
}

macro_rules! impl_from {
    ($variant:path, $for_type:ty) => {
        impl<Float: FloatPolicy> From<$for_type> for Key<Float> {
            fn from(v: $for_type) -> Key<Float> {
                $variant(v.into())
            }
        }
    };
}

impl_integer_from!(I8, i8);
impl_integer_from!(I16, i16);
impl_integer_from!(I32, i32);
impl_integer_from!(I64, i64);
impl_integer_from!(I128, i128);
impl_integer_from!(U8, u8);
impl_integer_from!(U16, u16);
impl_integer_from!(U32, u32);
impl_integer_from!(U64, u64);
impl_integer_from!(U128, u128);

impl_float_try_from!(F32, f32);
impl_float_try_from!(F64, f64);

impl_from!(Key::Bool, bool);
impl_from!(Key::Bytes, Vec<u8>);
impl_from!(Key::String, String);
impl_from!(Key::Vec, Vec<Key<Float>>);
impl_from!(Key::Map, Vec<(Key<Float>, Key<Float>)>);

#[cfg(test)]
mod tests {
    use super::Key;

    #[test]
    fn assert_default() {
        assert_eq!(Key::Unit, Key::default());
    }
}
