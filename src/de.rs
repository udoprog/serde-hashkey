//! Deserialization for serde-hashkey.

use serde::de::{self, IntoDeserializer};

use crate::error::Error;
use crate::float::{FloatPolicy, FloatRepr};
use crate::key::{Float, Integer, Key};

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

impl<'de, F> IntoDeserializer<'de, Error> for &'de Key<F>
where
    F: FloatPolicy,
{
    type Deserializer = Deserializer<'de, F>;

    fn into_deserializer(self) -> Self::Deserializer {
        Deserializer::new(self)
    }
}

pub struct Deserializer<'de, F>
where
    F: FloatPolicy,
{
    value: &'de Key<F>,
}

impl<'de, F> Deserializer<'de, F>
where
    F: FloatPolicy,
{
    pub fn new(value: &'de Key<F>) -> Self {
        Self { value }
    }
}

impl<'de, F> de::Deserializer<'de> for Deserializer<'de, F>
where
    F: FloatPolicy,
{
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
            Key::Float(Float::F32(float)) => <F::F32 as FloatRepr<f32>>::visit(float, visitor),
            Key::Float(Float::F64(float)) => <F::F64 as FloatRepr<f64>>::visit(float, visitor),
            Key::String(s) => visitor.visit_str(s),
            Key::Seq(array) => visitor.visit_seq(SeqDeserializer::new(array)),
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

struct EnumDeserializer<'de, F>
where
    F: FloatPolicy,
{
    variant: &'de Key<F>,
    value: Option<&'de Key<F>>,
}

impl<'de, F> de::EnumAccess<'de> for EnumDeserializer<'de, F>
where
    F: FloatPolicy,
{
    type Error = Error;
    type Variant = VariantDeserializer<'de, F>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer<'de, F>
where
    F: FloatPolicy,
{
    value: Option<&'de Key<F>>,
}

impl<'de, F> de::VariantAccess<'de> for VariantDeserializer<'de, F>
where
    F: FloatPolicy,
{
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
            Some(Key::Seq(values)) => {
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

struct SeqDeserializer<'de, F>
where
    F: FloatPolicy,
{
    values: &'de [Key<F>],
}

impl<'de, F> SeqDeserializer<'de, F>
where
    F: FloatPolicy,
{
    pub fn new(values: &'de [Key<F>]) -> Self {
        Self { values }
    }
}

impl<'de, F> serde::Deserializer<'de> for SeqDeserializer<'de, F>
where
    F: FloatPolicy,
{
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

        if self.values.is_empty() {
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

impl<'de, F> de::SeqAccess<'de> for SeqDeserializer<'de, F>
where
    F: FloatPolicy,
{
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

struct MapDeserializer<'de, F>
where
    F: FloatPolicy,
{
    map: &'de [(Key<F>, Key<F>)],
    value: Option<&'de Key<F>>,
}

impl<'de, F> MapDeserializer<'de, F>
where
    F: FloatPolicy,
{
    pub fn new(map: &'de [(Key<F>, Key<F>)]) -> Self {
        Self { map, value: None }
    }
}

impl<'de, F> serde::Deserializer<'de> for MapDeserializer<'de, F>
where
    F: FloatPolicy,
{
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

impl<'de, F> de::MapAccess<'de> for MapDeserializer<'de, F>
where
    F: FloatPolicy,
{
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
