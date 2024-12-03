use serde::{
    de::{self, Visitor},
    ser::{Serialize, SerializeTuple},
    Deserialize,
};

use crate::micro_cdr;

pub const SUBHEADER_SIZE: usize = 4;

///
/// 0       4       8               16               24               31
/// +-------+-------+----------------+----------------+----------------+
/// |  submessageId |       flags    |         submessageLength        |
/// +-------+-------+----------------+----------------+----------------+
///
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubMessageHeader {
    CreateClient(u16),
    Create(u16, bool, bool),
    GetInfo(u16),
    Delete(u16),
    StatusAgent(u16),
    Status(u16),
    Info(u16),
    WriteData(u16, DataFormat),
    ReadData(u16),
    Data(u16, DataFormat),
    AckNack(u16),
    HeartBeat(u16),
    Reset(u16),
    Fragment(u16, bool),
    TimeStamp(u16),
    TimeStampReply(u16),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataFormat {
    FormatData = 0x00,
    FormatSample = 0x01,
    FormatDataSeq = 0x04,
    FormatSampleSeq = 0x05,
    FormatPackedSamples = 0x07,
}

impl TryFrom<u8> for DataFormat {
    type Error = crate::error::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::FormatData),
            0x01 => Ok(Self::FormatSample),
            0x04 => Ok(Self::FormatDataSeq),
            0x05 => Ok(Self::FormatSampleSeq),
            0x07 => Ok(Self::FormatPackedSamples),
            _ => Err(crate::error::Error::InvalidFormat(value)),
        }
    }
}

impl Serialize for SubMessageHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use SubMessageHeader::*;

        let (id, flags, length) = match *self {
            CreateClient(len) => (0u8, 0u8, len),
            Create(len, replace, reuse) => (
                1u8,
                ((replace as u8) << 2) | ((reuse as u8) << 1) | 0u8,
                len,
            ),
            GetInfo(len) => (2u8, 0u8, len),
            Delete(len) => (3u8, 0u8, len),
            StatusAgent(len) => (4u8, 0, len),
            Status(len) => (5u8, 0, len),
            Info(len) => (6u8, 0, len),
            WriteData(len, format) => (7u8, ((format as u8) << 1) | 0, len),
            ReadData(len) => (8u8, 0, len),
            Data(len, format) => (9u8, ((format as u8) << 1) | 0, len),
            AckNack(len) => (10u8, 0, len),
            HeartBeat(len) => (11u8, 0, len),
            Reset(len) => (12u8, 0, len),
            Fragment(len, last) => (13u8, ((last as u8) << 1) | 0u8, len),
            TimeStamp(len) => (14u8, 0, len),
            TimeStampReply(len) => (15u8, 0, len),
        };

        let flags = flags
            | match micro_cdr::NATIVE_ENDIANNESS {
                crate::Endianness::LittleEndianness => 1u8,
                crate::Endianness::BigEndianness => 0u8,
            };

        let mut s = serializer.serialize_tuple(0)?;
        s.serialize_element(&id)?;
        s.serialize_element(&flags)?;
        s.serialize_element(&length)?;

        s.end()
    }
}

impl<'de> Deserialize<'de> for SubMessageHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use SubMessageHeader::*;

        struct SubMessageHeaderVistor;

        impl<'de> Visitor<'de> for SubMessageHeaderVistor {
            type Value = SubMessageHeader;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("struct SubMessageHeader")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let flag: u8 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let len: u16 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let result = match id {
                    0 => CreateClient(len),
                    1 => Create(len, (flag & 0b0000_0100) != 0, (flag & 0b0000_0010) != 0),
                    2 => GetInfo(len),
                    3 => Delete(len),
                    4 => StatusAgent(len),
                    5 => Status(len),
                    6 => Info(len),
                    7 => WriteData(
                        len,
                        DataFormat::try_from(flag >> 1).map_err(|e| match e {
                            crate::error::Error::InvalidFormat(v) => {
                                de::Error::invalid_value(de::Unexpected::Unsigned(v as u64), &self)
                            }
                            _ => unreachable!(),
                        })?,
                    ),
                    8 => ReadData(len),
                    9 => Data(
                        len,
                        DataFormat::try_from(flag >> 1).map_err(|e| match e {
                            crate::error::Error::InvalidFormat(v) => {
                                de::Error::invalid_value(de::Unexpected::Unsigned(v as u64), &self)
                            }
                            _ => unreachable!(),
                        })?,
                    ),
                    10 => AckNack(len),
                    11 => HeartBeat(len),
                    12 => Reset(len),
                    13 => Fragment(len, (flag & 0b0000_0010) != 0),
                    14 => TimeStamp(len),
                    15 => TimeStampReply(len),
                    _ => unreachable!(),
                };
                Ok(result)
            }
        }

        deserializer.deserialize_tuple_struct("", 3, SubMessageHeaderVistor)
    }
}

impl SubMessageHeader {
    pub fn to_slice(self, buf: &mut [u8]) -> crate::error::Result<usize> {
        let mut ucdr = micro_cdr::Encoder::new(buf);
        self.serialize(&mut ucdr)?;
        Ok(ucdr.finalize())
    }

    pub fn from_slice(buf: &[u8]) -> crate::error::Result<SubMessageHeader> {
        let mut ucdr = micro_cdr::Decoder::new(buf);
        SubMessageHeader::deserialize(&mut ucdr)
    }
}
