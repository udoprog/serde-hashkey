//! In-memory value representation for values.
use ordered_float::OrderedFloat;
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Float {
    /// Variant for an `f32`, in a wrapper implementing a total ordering.
    F32(OrderedFloat<f32>),
    /// Variant for an `f64`, in a wrapper implementing a total ordering.
    F64(OrderedFloat<f64>),
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
pub enum Key {
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
    Vec(Vec<Key>),
    /// A map.
    Map(Vec<(Key, Key)>),
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
        impl From<$for_type> for Key {
            fn from(v: $for_type) -> Key {
                Key::Integer(Integer::$variant(v))
            }
        }
    };
}

macro_rules! impl_float_from {
    ($variant:ident, $for_type:ty) => {
        impl From<$for_type> for Key {
            fn from(v: $for_type) -> Key {
                Key::Float(Float::$variant(OrderedFloat(v)))
            }
        }
    };
}

macro_rules! impl_from {
    ($variant:path, $for_type:ty) => {
        impl From<$for_type> for Key {
            fn from(v: $for_type) -> Key {
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

impl_float_from!(F32, f32);
impl_float_from!(F64, f64);

impl_from!(Key::Bool, bool);
impl_from!(Key::Bytes, Vec<u8>);
impl_from!(Key::String, String);
impl_from!(Key::Vec, Vec<Key>);
impl_from!(Key::Map, Vec<(Key, Key)>);

#[cfg(test)]
mod tests {
    use super::Key;

    #[test]
    fn assert_default() {
        assert_eq!(Key::Unit, Key::default());
    }
}
