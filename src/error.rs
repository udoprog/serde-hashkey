//! Errors raised during serialization/deserialization.
use serde::{de, ser};
use std::{error, fmt, result};

/// Errors that can occur during serialization and deserialization of a
/// [Key](crate::Key).
#[derive(Debug)]
pub enum Error {
    /// Unexpected type encountered.
    Unexpected(&'static str),
    /// Type is not supported for serialization.
    UnsupportedType(&'static str),
    /// Unsupported deserialization variant.
    UnexpectedVariant(&'static str),
    /// A custom error.
    Custom(String),
    /// Value is missing during deserialization.
    MissingValue,
    /// Array has invalid length.
    InvalidLength,
}

/// Helper alias for a Result which already represents our local [Error] type.
pub type Result<T, E = Error> = result::Result<T, E>;

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Error::*;

        match self {
            Unexpected(expected) => write!(fmt, "unexpected type, expected: {}", expected),
            UnsupportedType(ty) => write!(fmt, "unsupported type: {}", ty),
            UnexpectedVariant(variant) => write!(fmt, "unexpectec variant: {}", variant),
            Custom(e) => write!(fmt, "{}", e),
            MissingValue => write!(fmt, "missing value duration deserialization"),
            InvalidLength => write!(fmt, "array with invalid length"),
        }
    }
}

impl error::Error for Error {}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}
