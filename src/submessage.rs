

use serde::ser::{Serialize, SerializeTuple};

use crate::micro_cdr;

pub const SUBHEADER_SIZE:usize = 4;

///
/// 0       4       8               16               24               31
/// +-------+-------+----------------+----------------+----------------+
/// |  submessageId |       flags    |         submessageLength        |
/// +-------+-------+----------------+----------------+----------------+
/// 
#[allow(dead_code)]
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
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    FormatData = 0x00,
    FormatSample = 0x01,
    FormatDataSeq = 0x04,
    FormatSampleSeq = 0x05,
    FormatPackedSamples = 0x07,
}

impl Serialize for SubMessageHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        use SubMessageHeader::*;

        let (id, flags, length) = match *self {
            CreateClient(len) => (0u8, 0u8, len),
            Create(len, replace, reuse) => (
                1u8, ((replace as u8) << 2) | ((reuse as u8) << 1) | 0u8 , len),
            GetInfo(len) => (2u8, 0u8, len),
            Delete(len) => (3u8, 0u8, len),
            StatusAgent(len) => (4u8, 0, len),
            Status(len) => (5u8, 0, len),
            Info(len) => (6u8, 0, len),
            WriteData(len, format) => (
                7u8, ((format as u8) << 1) | 0, len
            ),
            ReadData(len) => (8u8, 0, len),
            Data(len, format) => (
                9u8, ((format as u8) << 1) | 0, len 
            ),
            AckNack(len) => (10u8, 0, len),
            HeartBeat(len) => (11u8, 0, len),
            Reset(len) => (12u8, 0, len),
            Fragment(len, last) => (13u8, ((last as u8) << 1) | 0u8, len),
            TimeStamp(len) => (14u8, 0, len),
            TimeStampReply(len) => (15u8, 0, len),
        };

        let flags = flags | match micro_cdr::NATIVE_ENDIANNESS {
            crate::Endianness::LittleEndianness => 1u8,
            crate::Endianness::BigEndianness => 0u8,
        } ;

        let mut s = serializer.serialize_tuple(0)?;
        s.serialize_element(&id)?;
        s.serialize_element(&flags)?;
        s.serialize_element(&length)?;

        s.end()
    }
}
