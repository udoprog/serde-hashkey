//! Serialization for serde-hashkey.

use crate::error::Error;
use serde::ser;
use std::marker::PhantomData;

use crate::float::{FloatPolicy, FloatRepr, RejectFloatPolicy};
use crate::key::{Float, Key};

/// Serialize the given value to a [Key].
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
pub fn to_key<T>(value: &T) -> Result<Key<RejectFloatPolicy>, Error>
where
    T: ser::Serialize,
{
    to_key_with_policy::<T, RejectFloatPolicy>(value)
}

/// Internal helper to serialize a value with the given policy.
pub(crate) fn to_key_with_policy<T, F>(value: &T) -> Result<Key<F>, Error>
where
    T: ser::Serialize,
    F: FloatPolicy,
{
    value.serialize(Serializer(PhantomData))
}

struct Serializer<F>(PhantomData<F>)
where
    F: FloatPolicy;

impl<F> ser::Serializer for Serializer<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    type SerializeSeq = SerializeVec<F>;
    type SerializeTuple = SerializeVec<F>;
    type SerializeTupleStruct = SerializeVec<F>;
    type SerializeTupleVariant = SerializeTupleVariant<F>;
    type SerializeMap = SerializeMap<F>;
    type SerializeStruct = SerializeMap<F>;
    type SerializeStructVariant = SerializeStructVariant<F>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Key<F>, Error> {
        Ok(Key::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    fn serialize_i128(self, value: i128) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<Key<F>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Key<F>, Error> {
        Ok(Key::Float(Float::F32(
            <F::F32 as FloatRepr<f32>>::serialize(value)?,
        )))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Key<F>, Error> {
        Ok(Key::Float(Float::F64(
            <F::F64 as FloatRepr<f64>>::serialize(value)?,
        )))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Key<F>, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Key<F>, Error> {
        Ok(Key::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Key<F>, Error> {
        Ok(Key::Bytes(value.to_vec()))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Key<F>, Error> {
        Ok(Key::Unit)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Key<F>, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Key<F>, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Key<F>, Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Key<F>, Error>
    where
        T: ser::Serialize,
    {
        let value = (Key::from(variant.to_owned()), to_key_with_policy(&value)?);
        Ok(Key::Map(vec![value]))
    }

    #[inline]
    fn serialize_none(self) -> Result<Key<F>, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Key<F>, Error>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Ok(SerializeMap {
            map: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: Vec::new(),
        })
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

pub struct SerializeVec<F>
where
    F: FloatPolicy,
{
    vec: Vec<Key<F>>,
}

pub struct SerializeTupleVariant<F>
where
    F: FloatPolicy,
{
    name: String,
    vec: Vec<Key<F>>,
}

pub struct SerializeMap<F>
where
    F: FloatPolicy,
{
    map: Vec<(Key<F>, Key<F>)>,
    next_key: Option<Key<F>>,
}

pub struct SerializeStructVariant<F>
where
    F: FloatPolicy,
{
    name: String,
    map: Vec<(Key<F>, Key<F>)>,
}

impl<F> ser::SerializeSeq for SerializeVec<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(to_key_with_policy(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Key<F>, Error> {
        Ok(Key::Vec(self.vec))
    }
}

impl<F> ser::SerializeTuple for SerializeVec<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Key<F>, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<F> ser::SerializeTupleStruct for SerializeVec<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Key<F>, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<F> ser::SerializeTupleVariant for SerializeTupleVariant<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(to_key_with_policy(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Key<F>, Error> {
        let value = (Key::from(self.name), Key::Vec(self.vec));
        Ok(Key::Map(vec![value]))
    }
}

impl<F> ser::SerializeMap for SerializeMap<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.next_key = Some(to_key_with_policy(&key)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        let key = match self.next_key.take() {
            Some(key) => key,
            None => return Err(Error::MissingValue),
        };

        self.map.push((key, to_key_with_policy(&value)?));
        Ok(())
    }

    fn end(self) -> Result<Key<F>, Error> {
        Ok(Key::Map(self.map))
    }
}

impl<F> ser::SerializeStruct for SerializeMap<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Key<F>, Error> {
        ser::SerializeMap::end(self)
    }
}

impl<F> ser::SerializeStructVariant for SerializeStructVariant<F>
where
    F: FloatPolicy,
{
    type Ok = Key<F>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.map
            .push((Key::from(String::from(key)), to_key_with_policy(&value)?));
        Ok(())
    }

    fn end(self) -> Result<Key<F>, Error> {
        let value = (Key::from(self.name), Key::Map(self.map));
        Ok(Key::Map(vec![value]))
    }
}
