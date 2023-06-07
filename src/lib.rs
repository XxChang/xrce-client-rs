#![no_std]
#![no_main]

pub mod error;
pub mod micro_cdr;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    BigEndianness,
    LittleEndianness,
}



