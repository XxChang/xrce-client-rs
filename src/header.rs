use serde::Serialize;
use serde::ser::SerializeTuple;
use crate::{error, micro_cdr};

pub const CLIENT_KEY_SIZE: usize = 4;
pub const SESSION_ID_WITHOUT_CLIENT_KEY: u8 = 0x80;


type ClientKey = [u8;4];

///
/// 0                8                16              24               31
/// +----------------+--------+-------+----------------+----------------+
/// |    sessionId   |     streamId   |            sequenceNr           |
/// +----------------+----------------+----------------+----------------+
/// |                  clientKey (if sessionId <= 127)                  |
/// +----------------+--------+-------+----------------+----------------+ 
pub struct MessageHeader {
    session_id: u8,
    stream_id: u8,
    sequence_num: u16,
    key: Option<ClientKey>,
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

impl MessageHeader {
    pub fn new(session_id: u8, stream_id: u8, seq_num: u16, key: Option<ClientKey>) -> Self {
        MessageHeader { 
            session_id, 
            stream_id, 
            sequence_num: seq_num, 
            key 
        }
    }

    pub fn to_slice(self, buf: &mut [u8]) -> error::Result<()> {
        let mut ucdr = micro_cdr::Encoder::new(buf);
        self.serialize(&mut ucdr)?;
        Ok(())
    }
}

