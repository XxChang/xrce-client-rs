use crate::error;
use crate::header::{CLIENT_KEY_SIZE, SESSION_ID_WITHOUT_CLIENT_KEY, MessageHeader};
use crate::submessage::SUBHEADER_SIZE;
use crate::types::{CREATE_CLIENT_Payload, CLIENT_Representation};

#[cfg(any(feature = "hard-liveliness-check", feature = "profile-shared-memory"))]
use crate::types::Property;

#[cfg(feature = "profile-shared-memory")]
const PROFILE_SHARED_MEMORY_ADD_SIZE:usize = 21;

#[cfg(not(feature = "profile-shared-memory"))]
const PROFILE_SHARED_MEMORY_ADD_SIZE:usize = 0;

#[cfg(feature = "hard-liveliness-check")]
const HARD_LIVELINESS_CHECK_ADD_SIZE: usize = 26;

#[cfg(not(feature = "hard-liveliness-check"))]
const HARD_LIVELINESS_CHECK_ADD_SIZE: usize = 0;

const CREATE_SESSION_PROPERTIES_MAX_SIZE: usize = PROFILE_SHARED_MEMORY_ADD_SIZE + HARD_LIVELINESS_CHECK_ADD_SIZE ;

const MIN_HEADER_SIZE: usize = 4;
const CREATE_CLIENT_PAYLOAD_SIZE: usize = 16;
const MAX_HEADER_SIZE: usize =  MIN_HEADER_SIZE + CLIENT_KEY_SIZE ;
const CREATE_SESSION_MAX_MSG_SIZE: usize = MAX_HEADER_SIZE + SUBHEADER_SIZE + CREATE_CLIENT_PAYLOAD_SIZE + CREATE_SESSION_PROPERTIES_MAX_SIZE;

type ClientKey = [u8;4];
///
/// 
struct SessionInfo {
    id: u8,
    key: ClientKey,
}

pub struct Session {
    info: SessionInfo,
    mtu: u16,
}

pub enum Error {

}

type SessionResult = core::result::Result<(), Error> ;

impl Session {
    pub fn new(key: ClientKey) -> Self {
        Session {
            info: SessionInfo { 
                id: 0x81,
                key,
            },
            mtu: 256,
        }
    }

    pub fn create(&mut self) -> SessionResult {
        let mut create_session_buffer = [0u8;CREATE_SESSION_MAX_MSG_SIZE] ;

        // indicate that there is no session and that the client_key does not follow the message
        MessageHeader::new(
            self.info.id & SESSION_ID_WITHOUT_CLIENT_KEY, 
            0, 0, 
            None).to_slice(&mut create_session_buffer[..MIN_HEADER_SIZE]).unwrap();
        
        self.buffer_create_session(self.mtu - core::mem::size_of::<usize>() as u16, &mut create_session_buffer[MIN_HEADER_SIZE..]).unwrap();
        Ok(())
    }

    fn buffer_create_session(&self, mtu: u16, buf: &mut [u8]) -> error::Result<()> {
        let payload = CREATE_CLIENT_Payload {
            client_representation: CLIENT_Representation {
                xrce_cookie: [b'X', b'R', b'C', b'E'],
                xrce_version: [0x01u8, 0x00u8],
                xrce_vendor_id: [0x01, 0x0F],
                client_key: self.info.key,
                session_id: self.info.id,

                #[cfg(all(not(feature = "hard-liveliness-check"), not(feature = "profile-shared-memory")))]
                properties: None,

                #[cfg(all(not(feature = "hard-liveliness-check"), feature = "profile-shared-memory"))]
                properties: Some(
                    [
                        Property {
                            name: "uxr_sm",
                            value: "1",
                        }
                    ]
                ),

                #[cfg(all(feature = "hard-liveliness-check", not(feature = "profile-shared-memory")))]
                properties: Some(
                    [
                        Property {
                            name: "uxr_hl",
                            value: "999999",
                        }
                    ]
                ),

                #[cfg(all(feature = "hard-liveliness-check", feature = "profile-shared-memory"))]
                properties: Some(
                    [
                        Property {
                            name: "uxr_sm",
                            value: "1",
                        },

                        Property {
                            name: "uxr_hl",
                            value: "999999",
                        }
                    ]
                ),

                mtu
            }
        };
        
        payload.to_slice(buf)
    }
}
