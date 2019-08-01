//! In-memory value representation for values.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Integer {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Unit,
    Bool(bool),
    Integer(Integer),
    Bytes(Vec<u8>),
    String(String),
    Vec(Vec<Key>),
    Map(Vec<(Key, Key)>),
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

impl_from!(Key::Bool, bool);
impl_from!(Key::Bytes, Vec<u8>);
impl_from!(Key::String, String);
impl_from!(Key::Vec, Vec<Key>);
impl_from!(Key::Map, Vec<(Key, Key)>);
