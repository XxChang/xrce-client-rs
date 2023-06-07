use serde::ser ;

pub type Result<T> = core::result::Result<T, Error> ;

#[derive(Debug)]
pub enum Error {
    CustomerMessage,
    BufferNotEnough,
    NumberOutOfRange,
    InvalidChar(char),
    InvalidString,
    SequenceMustHaveLength,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Error::*;

        match *self {
            CustomerMessage => write!(f, "Error!!!!"),
            BufferNotEnough => write!(f, "Buffer Length is Not Enough!"),
            NumberOutOfRange => write!(f, "sequence is too long"),
            InvalidChar(v) => write!(f, "expected char of width 1. found {}", v),
            InvalidString => write!(f, "each character must have a length of 1"),
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