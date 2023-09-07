use serde::Serialize;
use serde::Deserialize;
use serde::de;
use serde::de::Visitor;
use serde::ser::SerializeTuple;
use crate::{error, micro_cdr};

pub const CLIENT_KEY_SIZE: usize = 4;
pub const SESSION_ID_WITHOUT_CLIENT_KEY: u8 = 0x80;


type ClientKey = [u8;4];

///
/// 0                8               16               24               31
/// +----------------+--------+-------+----------------+----------------+
/// |    sessionId   |     streamId   |            sequenceNr           |
/// +----------------+----------------+----------------+----------------+
/// |                  clientKey (if sessionId <= 127)                  |
/// +----------------+--------+-------+----------------+----------------+ 
#[derive(Debug, PartialEq)]
pub struct MessageHeader {
    pub session_id: u8,
    pub stream_id: u8,
    pub sequence_num: u16,
    pub key: Option<ClientKey>,
}

impl Serialize for MessageHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
    
        let mut s = serializer.serialize_tuple(0)?;
        s.serialize_element(&self.session_id)?;
        s.serialize_element(&self.stream_id)?;
        s.serialize_element(&self.sequence_num)?;
        if let Some(key) = self.key {
            s.serialize_element(&key[0])?;
            s.serialize_element(&key[1])?;
            s.serialize_element(&key[2])?;
            s.serialize_element(&key[3])?;
        }
        s.end()
    }
}

impl<'de> Deserialize<'de> for MessageHeader {
    
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> 
    {
        struct MessageHeaderVisitor;

        impl<'de> Visitor<'de> for MessageHeaderVisitor {
            type Value = MessageHeader;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("struct MessageHeader")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: de::SeqAccess<'de>, {
                let session_id: u8 = seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let stream_id: u8 = seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let seq_num: u16 = seq.next_element()?
                        .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let key = if session_id < SESSION_ID_WITHOUT_CLIENT_KEY {
                    let k1: u8 = seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                    let k2: u8 = seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                    let k3: u8 = seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                    let k4: u8 = seq.next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                    Some([k1, k2, k3, k4])
                } else {
                    None
                };
                Ok(
                    MessageHeader { session_id, stream_id, sequence_num: seq_num, key }
                )
            }
        }

        deserializer.deserialize_tuple_struct("", 7, MessageHeaderVisitor)
    }
}

impl MessageHeader {
    pub fn new(session_id: u8, stream_id: u8, seq_num: u16, key: Option<ClientKey>) -> Self {
        MessageHeader { 
            session_id, 
            stream_id, 
            sequence_num: seq_num, 
            key 
        }
    }

    pub fn to_slice(self, buf: &mut [u8]) -> error::Result<usize> {
        let mut ucdr = micro_cdr::Encoder::new(buf);
        self.serialize(&mut ucdr)?;
        Ok(ucdr.finalize())
    }

    pub fn from_slice(buf: &[u8]) -> error::Result<MessageHeader> {
        let mut ucdr = micro_cdr::Decoder::new(buf);
        MessageHeader::deserialize(&mut ucdr)
    }
}

