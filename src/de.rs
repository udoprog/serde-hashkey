//! Deserialization for serde-hashkey.

use serde::de::{self, IntoDeserializer};
use std::fmt;
use std::marker::PhantomData;

use crate::error::Error;
use crate::float::FloatPolicy;
use crate::key::{Integer, Key};

/// Deserialize the given type from a [Key].
///
/// # Examples
///
/// ```rust
/// use serde_derive::{Deserialize, Serialize};
/// use serde_hashkey::{from_key, to_key, Key};
/// use std::collections::HashMap;
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// struct Author {
///     name: String,
///     age: u32,
/// }
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
///         age: 42,
///     },
/// };
///
/// let key = to_key(&book)?;
/// let book2 = from_key(&key)?;
///
/// assert_eq!(book, book2);
/// # Ok(())
/// # }
/// ```
///
/// Using a non-standard float policy:
///
/// ```rust
/// use serde_derive::{Deserialize, Serialize};
/// use serde_hashkey::{from_key, to_key_with_ordered_float, OrderedFloat, Key};
/// use std::collections::HashMap;
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
/// struct Author {
///     name: String,
///     age: u32,
/// }
///
/// #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
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
///         age: 42,
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
pub fn from_key<T, F>(value: &Key<F>) -> Result<T, crate::error::Error>
where
    T: de::DeserializeOwned,
    F: FloatPolicy,
{
    T::deserialize(Deserializer::new(&value))
}

impl<'de, Float: FloatPolicy> IntoDeserializer<'de, Error> for &'de Key<Float> {
    type Deserializer = Deserializer<'de, Float>;

    fn into_deserializer(self) -> Self::Deserializer {
        Deserializer::new(self)
    }
}

pub struct Deserializer<'de, Float: FloatPolicy> {
    value: &'de Key<Float>,
}

impl<'de, Float: FloatPolicy> Deserializer<'de, Float> {
    pub fn new(value: &'de Key<Float>) -> Self {
        Self { value }
    }
}

impl<'de, Float: FloatPolicy> de::Deserializer<'de> for Deserializer<'de, Float> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        return match self.value {
            Key::Unit => visitor.visit_unit(),
            Key::Bool(b) => visitor.visit_bool(*b),
            Key::Integer(Integer::U8(v)) => visitor.visit_u8(*v),
            Key::Integer(Integer::U16(v)) => visitor.visit_u16(*v),
            Key::Integer(Integer::U32(v)) => visitor.visit_u32(*v),
            Key::Integer(Integer::U64(v)) => visitor.visit_u64(*v),
            Key::Integer(Integer::U128(v)) => visitor.visit_u128(*v),
            Key::Integer(Integer::I8(v)) => visitor.visit_i8(*v),
            Key::Integer(Integer::I16(v)) => visitor.visit_i16(*v),
            Key::Integer(Integer::I32(v)) => visitor.visit_i32(*v),
            Key::Integer(Integer::I64(v)) => visitor.visit_i64(*v),
            Key::Integer(Integer::I128(v)) => visitor.visit_i128(*v),
            Key::Float(float) => float.deserialize_float(visitor),
            Key::String(s) => visitor.visit_str(s),
            Key::Vec(array) => visitor.visit_seq(SeqDeserializer::new(array)),
            Key::Map(m) => visitor.visit_map(MapDeserializer::new(m)),
            Key::Bytes(bytes) => visitor.visit_borrowed_bytes(bytes),
        };
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Key::Unit => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        let (variant, value) = match self.value {
            Key::Map(value) => {
                let mut iter = value.iter();

                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(Error::Unexpected("map with a single key"));
                    }
                };

                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(Error::Unexpected("map with a single key"));
                }

                (variant, Some(value))
            }
            Key::String(_) => (self.value, None),
            _ => {
                return Err(Error::Unexpected("string or map"));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
        bytes byte_buf
    }
}

struct EnumDeserializer<'de, Float: FloatPolicy> {
    variant: &'de Key<Float>,
    value: Option<&'de Key<Float>>,
}

impl<'de, Float: FloatPolicy> de::EnumAccess<'de> for EnumDeserializer<'de, Float> {
    type Error = Error;
    type Variant = VariantDeserializer<'de, Float>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer<'de, Float: FloatPolicy> {
    value: Option<&'de Key<Float>>,
}

impl<'de, Float: FloatPolicy> de::VariantAccess<'de> for VariantDeserializer<'de, Float> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(Deserializer::new(value)),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(Deserializer::new(value)),
            None => Err(Error::UnexpectedVariant("newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Key::Vec(values)) => {
                de::Deserializer::deserialize_any(SeqDeserializer::new(values), visitor)
            }
            Some(_) => Err(Error::UnexpectedVariant("tuple variant")),
            None => Err(Error::UnexpectedVariant("tuple variant")),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Key::Map(v)) => {
                de::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(_) => Err(Error::UnexpectedVariant("struct variant")),
            _ => Err(Error::UnexpectedVariant("struct variant")),
        }
    }
}

struct SeqDeserializer<'de, Float: FloatPolicy> {
    values: &'de [Key<Float>],
}

impl<'de, Float: FloatPolicy> SeqDeserializer<'de, Float> {
    pub fn new(values: &'de [Key<Float>]) -> Self {
        Self { values }
    }
}

impl<'de, Float: FloatPolicy> serde::Deserializer<'de> for SeqDeserializer<'de, Float> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        let len = self.values.len();

        if len == 0 {
            return visitor.visit_unit();
        }

        let ret = visitor.visit_seq(&mut self)?;

        if self.values.len() == 0 {
            return Ok(ret);
        }

        Err(Error::InvalidLength)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, Float: FloatPolicy> de::SeqAccess<'de> for SeqDeserializer<'de, Float> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let (first, rest) = match self.values.split_first() {
            Some((first, rest)) => (first, rest),
            None => return Ok(None),
        };

        self.values = rest;
        let value = seed.deserialize(Deserializer::new(first))?;
        Ok(Some(value))
    }
}

struct MapDeserializer<'de, Float: FloatPolicy> {
    map: &'de [(Key<Float>, Key<Float>)],
    value: Option<&'de Key<Float>>,
}

impl<'de, Float: FloatPolicy> MapDeserializer<'de, Float> {
    pub fn new(map: &'de [(Key<Float>, Key<Float>)]) -> Self {
        Self { map, value: None }
    }
}

impl<'de, Float: FloatPolicy> serde::Deserializer<'de> for MapDeserializer<'de, Float> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, Float: FloatPolicy> de::MapAccess<'de> for MapDeserializer<'de, Float> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let next = self.map.split_first();

        match next {
            Some(((key, value), map)) => {
                self.value = Some(value);
                self.map = map;
                let value = seed.deserialize(key.into_deserializer())?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = match self.value.take() {
            Some(value) => value,
            None => return Err(Error::MissingValue),
        };

        seed.deserialize(Deserializer::new(value))
    }
}

impl<'de, Float: FloatPolicy> de::Deserialize<'de> for Key<Float> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Key<Float>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueVisitor<Float: FloatPolicy>(PhantomData<Float>);

        impl<'de, Float: FloatPolicy> de::Visitor<'de> for ValueVisitor<Float> {
            type Value = Key<Float>;

            fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.write_str("any valid key")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::String(value))
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
                Ok(Key::Bytes(v))
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
                let mut vec = Vec::new();

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Key::Vec(vec))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(Key::Map(values))
            }
        }

        deserializer.deserialize_any(ValueVisitor::<Float>(PhantomData))
    }
}
