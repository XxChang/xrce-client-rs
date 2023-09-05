use core::str::Utf8Error;

use serde::ser ;
use serde::de ;

pub type Result<T> = core::result::Result<T, Error> ;

#[derive(Debug)]
pub enum Error {
    CustomerMessage,
    BufferNotEnough,
    InvalidBoolEncoding(u8),
    NumberOutOfRange,
    InvalidChar(char),
    InvalidCharEncoding,
    InvalidFormat(u8),
    InvalidUtf8Encoding(Utf8Error),
    InvalidString,
    SequenceMustHaveLength,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;

        match *self {
            CustomerMessage => write!(f, "Error!!!!"),
            BufferNotEnough => write!(f, "Buffer Length is Not Enough!"),
            InvalidBoolEncoding(v) => write!(f, "expected 0 or 1, found {}", v),
            NumberOutOfRange => write!(f, "sequence is too long"),
            InvalidChar(v) => write!(f, "expected char of width 1. found {}", v),
            InvalidCharEncoding => write!(f, "char is not valid UTF-8"),
            InvalidFormat(v) => write!(f, "invalid format {v}"),
            InvalidString => write!(f, "each character must have a length of 1"),
            InvalidUtf8Encoding(ref err) => core::fmt::Display::fmt(err, f),
            SequenceMustHaveLength => write!(f, "sequences must have a knowable size ahead of time"),
        }
    }
}

impl ser::Error for Error {
    fn custom<T>(_:T) -> Self 
    where 
        T: core::fmt::Display {
        Error::CustomerMessage
    }
}

impl de::Error for Error {
    fn custom<T>(_:T) -> Self where T:core::fmt::Display {
        Error::CustomerMessage
    }
}