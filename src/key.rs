//! In-memory value representation for values.
use crate::error::Error;
use serde::{de, ser};
use std::cmp::Ordering;
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

/// A float serialization policy which rejects any attempt to serialize a float with an error.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum RejectFloat {}

/// A policy for handling floating point types in a `Key`.
///
/// Currently there are two important `FloatPolicy` types: [`RejectFloat`] and
/// [`OrderedFloat`]. The former will emit errors instead of allowing floats to
/// be serialized and the latter while serialize them and provide a total order
/// which does not adhere to the IEEE standard.
///
/// [`RejectFloat`]: RejectFloat
/// [`OrderedFloat`]: OrderedFloat
pub trait FloatPolicy: Clone + PartialEq + Eq + PartialOrd + Ord + Hash {
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
