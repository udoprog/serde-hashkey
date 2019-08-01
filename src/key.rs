//! In-memory value representation for values.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Bool(bool),
    Integer(i128),
    Bytes(Vec<u8>),
    String(String),
    Vec(Vec<Key>),
    Map(Vec<(Key, Key)>),
    Option(Option<Box<Key>>),
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

impl_from!(Key::Bool, bool);
impl_from!(Key::Integer, i8);
impl_from!(Key::Integer, i16);
impl_from!(Key::Integer, i32);
impl_from!(Key::Integer, i64);
impl_from!(Key::Integer, u8);
impl_from!(Key::Integer, u16);
impl_from!(Key::Integer, u32);
impl_from!(Key::Integer, u64);
impl_from!(Key::Bytes, Vec<u8>);
impl_from!(Key::String, String);
impl_from!(Key::Vec, Vec<Key>);
impl_from!(Key::Map, Vec<(Key, Key)>);
