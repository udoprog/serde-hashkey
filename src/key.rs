//! In-memory value representation for values.
use crate::float::{FloatPolicy, FloatRepr, RejectFloatPolicy};
use serde::{de, ser};
use std::fmt;
use std::hash::Hash;
use std::marker;
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

/// An opaque float derived from a given policy.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Float<F>
where
    F: FloatPolicy,
{
    /// Variant representing a `f32` float.
    F32(F::F32),
    /// Variant representing a `f64` float.
    F64(F::F64),
}

/// The central key type, which is an in-memory representation of all supported
/// serde-serialized values.
///
/// This can be serialized to a type implementing [serde::Deserialize] using
/// [from_key], and deserialized from a type implementing [serde::Serialize]
/// using [to_key]. See the corresponding function for documentation.
///
/// The type parameter `F` corresponds to the [FloatPolicy] in used. It defaults
/// to [RejectFloatPolicy] which will cause floats to be rejected.
///
/// [from_key]: crate::from_key
/// [to_key]: crate::to_key
///
/// # Examples
///
/// ```
/// use serde_derive::{Deserialize, Serialize};
/// use serde_hashkey::{to_key, to_key_with_ordered_float};
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// struct Author {
///     name: String,
///     age: u32,
/// }
///
/// # fn main() -> Result<(), serde_hashkey::Error> {
/// let key = to_key(&Author {
///     name: String::from("Jane Doe"),
///     age: 42,
/// })?;
///
/// // Note: serializing floats will fail under the default policy, but succeed
/// // under one supporting floats.
/// assert!(to_key(&42.0f32).is_err());
/// assert!(to_key_with_ordered_float(&42.0f32).is_ok());
/// # Ok(()) }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key<F = RejectFloatPolicy>
where
    F: FloatPolicy,
{
    /// A unit value.
    Unit,
    /// A boolean value.
    Bool(bool),
    /// An integer.
    Integer(Integer),
    /// A 32-bit floating-point number.
    Float(Float<F>),
    /// A byte array.
    Bytes(Box<[u8]>),
    /// A string.
    String(Box<str>),
    /// A vector.
    Seq(Box<[Key<F>]>),
    /// A map.
    Map(Box<[(Key<F>, Key<F>)]>),
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
            Key::Seq(mut vec) => {
                for value in vec.iter_mut() {
                    *value = mem::replace(value, Key::Unit).normalize();
                }

                Key::Seq(vec)
            }
            Key::Map(mut map) => {
                for (key, value) in map.iter_mut() {
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
        impl<F> From<$for_type> for Key<F>
        where
            F: FloatPolicy,
        {
            fn from(v: $for_type) -> Key<F> {
                Key::Integer(Integer::$variant(v))
            }
        }
    };
}

macro_rules! impl_from {
    ($variant:path, $for_type:ty) => {
        impl<F> From<$for_type> for Key<F>
        where
            F: FloatPolicy,
        {
            fn from(v: $for_type) -> Key<F> {
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
impl_from!(Key::Seq, Vec<Key<F>>);
impl_from!(Key::Map, Vec<(Key<F>, Key<F>)>);

/// Serialize implementation for a [Key].
///
/// This allows keys to be serialized immediately.
///
/// # Examples
///
/// ```
/// use serde_derive::Serialize;
/// use serde_hashkey::{Key, OrderedFloatPolicy, OrderedFloat, Float};
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// struct Foo {
///     key: Key<OrderedFloatPolicy>,
/// }
///
/// # fn main() -> Result<(), serde_json::Error> {
/// let foo: String = serde_json::to_string(&Foo { key: Key::Float(Float::F64(OrderedFloat(42.42f64))) })?;
///
/// assert_eq!(foo, "{\"key\":42.42}");
/// Ok(())
/// # }
/// ```
impl<F> ser::Serialize for Key<F>
where
    F: FloatPolicy,
{
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            Key::Unit => serializer.serialize_unit(),
            Key::Integer(Integer::U8(v)) => serializer.serialize_u8(*v),
            Key::Integer(Integer::U16(v)) => serializer.serialize_u16(*v),
            Key::Integer(Integer::U32(v)) => serializer.serialize_u32(*v),
            Key::Integer(Integer::U64(v)) => serializer.serialize_u64(*v),
            Key::Integer(Integer::U128(v)) => serializer.serialize_u128(*v),
            Key::Integer(Integer::I8(v)) => serializer.serialize_i8(*v),
            Key::Integer(Integer::I16(v)) => serializer.serialize_i16(*v),
            Key::Integer(Integer::I32(v)) => serializer.serialize_i32(*v),
            Key::Integer(Integer::I64(v)) => serializer.serialize_i64(*v),
            Key::Integer(Integer::I128(v)) => serializer.serialize_i128(*v),
            Key::Float(Float::F32(float)) => float.serialize(serializer),
            Key::Float(Float::F64(float)) => float.serialize(serializer),
            Key::Bytes(v) => serializer.serialize_bytes(&v),
            Key::String(v) => serializer.serialize_str(&v),
            Key::Seq(v) => v.serialize(serializer),
            Key::Map(m) => {
                use self::ser::SerializeMap as _;

                let mut map = serializer.serialize_map(Some(m.len()))?;

                for (k, v) in m.iter() {
                    map.serialize_key(k)?;
                    map.serialize_value(v)?;
                }

                map.end()
            }
            Key::Bool(v) => serializer.serialize_bool(*v),
        }
    }
}

/// Deserialize implementation for a [Key].
///
/// This allows keys to be serialized immediately.
///
/// # Examples
///
/// ```
/// use serde_derive::Deserialize;
/// use serde_hashkey::{Key, OrderedFloatPolicy};
///
/// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
/// struct Foo {
///     key: Key<OrderedFloatPolicy>,
/// }
///
/// # fn main() -> Result<(), serde_json::Error> {
/// let foo: Foo = serde_json::from_str("{\"key\": 42.42}")?;
///
/// assert!(matches!(foo.key, Key::Float(..)));
/// Ok(())
/// # }
/// ```
impl<'de, F> de::Deserialize<'de> for Key<F>
where
    F: FloatPolicy,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Key<F>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueVisitor<F>(marker::PhantomData<F>)
        where
            F: FloatPolicy;

        impl<'de, F> de::Visitor<'de> for ValueVisitor<F>
        where
            F: FloatPolicy,
        {
            type Value = Key<F>;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("any valid key")
            }

            #[inline]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::String(s.into()))
            }

            #[inline]
            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::String(s.into()))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_byte_buf(v.to_owned())
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::Bytes(v.into()))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.into())
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::Float(Float::F32(
                    <F::F32 as FloatRepr<f32>>::serialize(v).map_err(E::custom)?,
                )))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::Float(Float::F64(
                    <F::F64 as FloatRepr<f64>>::serialize(v).map_err(E::custom)?,
                )))
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::Bool(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_unit()
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::Unit)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = visitor
                    .size_hint()
                    .map(Vec::with_capacity)
                    .unwrap_or_default();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Key::Seq(vec.into()))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = visitor
                    .size_hint()
                    .map(Vec::with_capacity)
                    .unwrap_or_default();

                while let Some((key, value)) = visitor.next_entry()? {
                    map.push((key, value));
                }

                Ok(Key::Map(map.into()))
            }
        }

        deserializer.deserialize_any(ValueVisitor::<F>(marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::RejectFloatPolicy;

    use super::Key;

    #[test]
    fn assert_default() {
        assert_eq!(Key::Unit, Key::default());
    }

    #[test]
    fn assert_impls() {
        assert_eq::<Key<RejectFloatPolicy>>();
        assert_hash::<Key<RejectFloatPolicy>>();
        assert_ord::<Key<RejectFloatPolicy>>();

        #[cfg(feature = "ordered-float")]
        {
            use crate::OrderedFloatPolicy;

            assert_eq::<Key<OrderedFloatPolicy>>();
            assert_hash::<Key<OrderedFloatPolicy>>();
            assert_ord::<Key<OrderedFloatPolicy>>();
        }

        fn assert_eq<T: std::cmp::Eq>() {}
        fn assert_hash<T: std::hash::Hash>() {}
        fn assert_ord<T: std::cmp::Ord>() {}
    }
}
