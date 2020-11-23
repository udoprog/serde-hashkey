//! Serialization for serde-hashkey.

use crate::error::Error;
use serde::ser;
use std::marker::PhantomData;

use crate::float::FloatPolicy;
use crate::key::{Integer, Key};

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
pub fn to_key<T>(value: &T) -> Result<Key<crate::RejectFloat>, Error>
where
    T: ser::Serialize,
{
    to_key_with_policy::<T, crate::RejectFloat>(value)
}

pub(crate) fn to_key_with_policy<T, F>(value: &T) -> Result<Key<F>, Error>
where
    T: ser::Serialize,
    F: FloatPolicy,
{
    value.serialize(Serializer(PhantomData))
}

impl<Float: FloatPolicy> ser::Serialize for Key<Float> {
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
            Key::Float(float) => float.serialize_float(serializer),
            Key::Bytes(v) => serializer.serialize_bytes(&v),
            Key::String(v) => serializer.serialize_str(&v),
            Key::Vec(v) => v.serialize(serializer),
            Key::Map(m) => {
                use self::ser::SerializeMap as _;

                let mut map = serializer.serialize_map(Some(m.len()))?;

                for (k, v) in m {
                    map.serialize_key(k)?;
                    map.serialize_value(v)?;
                }

                map.end()
            }
            Key::Bool(v) => serializer.serialize_bool(*v),
        }
    }
}

struct Serializer<Float: FloatPolicy>(PhantomData<Float>);

impl<Float: FloatPolicy> ser::Serializer for Serializer<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    type SerializeSeq = SerializeVec<Float>;
    type SerializeTuple = SerializeVec<Float>;
    type SerializeTupleStruct = SerializeVec<Float>;
    type SerializeTupleVariant = SerializeTupleVariant<Float>;
    type SerializeMap = SerializeMap<Float>;
    type SerializeStruct = SerializeMap<Float>;
    type SerializeStructVariant = SerializeStructVariant<Float>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Key<Float>, Error> {
        Ok(Key::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    fn serialize_i128(self, value: i128) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_u128(self, value: u128) -> Result<Key<Float>, Error> {
        Ok(value.into())
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Key<Float>, Error> {
        Float::serialize_f32(value).map(Key::Float)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Key<Float>, Error> {
        Float::serialize_f64(value).map(Key::Float)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Key<Float>, Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Key<Float>, Error> {
        Ok(Key::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Key<Float>, Error> {
        Ok(Key::Bytes(value.to_vec()))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Key<Float>, Error> {
        Ok(Key::Unit)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Key<Float>, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Key<Float>, Error> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Key<Float>, Error>
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
    ) -> Result<Key<Float>, Error>
    where
        T: ser::Serialize,
    {
        let value = (Key::from(variant.to_owned()), to_key_with_policy(&value)?);
        Ok(Key::Map(vec![value]))
    }

    #[inline]
    fn serialize_none(self) -> Result<Key<Float>, Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Key<Float>, Error>
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

pub struct SerializeVec<Float: FloatPolicy> {
    vec: Vec<Key<Float>>,
}

pub struct SerializeTupleVariant<Float: FloatPolicy> {
    name: String,
    vec: Vec<Key<Float>>,
}

pub struct SerializeMap<Float: FloatPolicy> {
    map: Vec<(Key<Float>, Key<Float>)>,
    next_key: Option<Key<Float>>,
}

pub struct SerializeStructVariant<Float: FloatPolicy> {
    name: String,
    map: Vec<(Key<Float>, Key<Float>)>,
}

impl<Float: FloatPolicy> ser::SerializeSeq for SerializeVec<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(to_key_with_policy(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Key<Float>, Error> {
        Ok(Key::Vec(self.vec))
    }
}

impl<Float: FloatPolicy> ser::SerializeTuple for SerializeVec<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Key<Float>, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<Float: FloatPolicy> ser::SerializeTupleStruct for SerializeVec<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Key<Float>, Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<Float: FloatPolicy> ser::SerializeTupleVariant for SerializeTupleVariant<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.vec.push(to_key_with_policy(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Key<Float>, Error> {
        let value = (Key::from(self.name), Key::Vec(self.vec));
        Ok(Key::Map(vec![value]))
    }
}

impl<Float: FloatPolicy> ser::SerializeMap for SerializeMap<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.next_key = Some(Key::from(to_key_with_policy(&key)?));
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

    fn end(self) -> Result<Key<Float>, Error> {
        Ok(Key::Map(self.map))
    }
}

impl<Float: FloatPolicy> ser::SerializeStruct for SerializeMap<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Key<Float>, Error> {
        ser::SerializeMap::end(self)
    }
}

impl<Float: FloatPolicy> ser::SerializeStructVariant for SerializeStructVariant<Float> {
    type Ok = Key<Float>;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ser::Serialize,
    {
        self.map
            .push((Key::from(String::from(key)), to_key_with_policy(&value)?));
        Ok(())
    }

    fn end(self) -> Result<Key<Float>, Error> {
        let value = (Key::from(self.name), Key::Map(self.map));
        Ok(Key::Map(vec![value]))
    }
}
