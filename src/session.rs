use super::Error;
use super::Result;
use crate::communication::{Receiver, Transmitter};
use crate::header::{MessageHeader, CLIENT_KEY_SIZE, SESSION_ID_WITHOUT_CLIENT_KEY};
use crate::micro_cdr::Decoder;
use crate::stream_id::StreamDirection;
use crate::stream_id::StreamId;
use crate::stream_id::StreamType;
use crate::submessage;
use crate::submessage::{SubMessageHeader, SUBHEADER_SIZE};
use crate::time::Clock;
use crate::types::{CLIENT_Representation, CREATE_CLIENT_Payload};
use crate::{error, micro_cdr, stream_id};
use crate::{MAX_SESSION_CONNECTION_ATTEMPTS, MIN_SESSION_CONNECTION_INTERVAL};
use core::convert::Infallible;
use core::marker::PhantomData;

#[cfg(any(feature = "hard-liveliness-check", feature = "profile-shared-memory"))]
use crate::types::Property;

#[cfg(feature = "profile-shared-memory")]
const PROFILE_SHARED_MEMORY_ADD_SIZE: usize = 21;

#[cfg(not(feature = "profile-shared-memory"))]
const PROFILE_SHARED_MEMORY_ADD_SIZE: usize = 0;

#[cfg(feature = "hard-liveliness-check")]
const HARD_LIVELINESS_CHECK_ADD_SIZE: usize = 26;

#[cfg(not(feature = "hard-liveliness-check"))]
const HARD_LIVELINESS_CHECK_ADD_SIZE: usize = 0;

const CREATE_SESSION_PROPERTIES_MAX_SIZE: usize =
    PROFILE_SHARED_MEMORY_ADD_SIZE + HARD_LIVELINESS_CHECK_ADD_SIZE;

pub const MIN_HEADER_SIZE: usize = 4;
const CREATE_CLIENT_PAYLOAD_SIZE: usize = 16;
pub const MAX_HEADER_SIZE: usize = MIN_HEADER_SIZE + CLIENT_KEY_SIZE;
const CREATE_SESSION_MAX_MSG_SIZE: usize = MAX_HEADER_SIZE
    + SUBHEADER_SIZE
    + CREATE_CLIENT_PAYLOAD_SIZE
    + CREATE_SESSION_PROPERTIES_MAX_SIZE;

type ClientKey = [u8; 4];
///
///
#[derive(Debug)]
struct SessionInfo {
    id: u8,
    key: ClientKey,
}

#[derive(Debug)]
pub struct Session<'storage, 'a: 'storage, Transport: Transmitter + Receiver<'a>> {
    transport: &'a mut Transport,
    info: SessionInfo,
    mtu: u16,
    _p: PhantomData<&'storage [u8]>,
}

type SessionResult<T> = core::result::Result<T, Error>;

impl<'storage, 'a: 'storage, T: Transmitter + Receiver<'a> + Clock> Session<'storage, 'a, T> {
    pub fn new(key: ClientKey, transport: &'a mut T) -> Self {
        Session {
            transport,
            info: SessionInfo { id: 0x81, key },
            mtu: 256,
            _p: PhantomData,
        }
    }

    pub fn create(&mut self) -> SessionResult<()> {
        let mut create_session_buffer = [0u8; CREATE_SESSION_MAX_MSG_SIZE];

        // indicate that there is no session and that the client_key does not follow the message
        let len1 = MessageHeader::new(self.info.id & SESSION_ID_WITHOUT_CLIENT_KEY, 0, 0, None)
            .to_slice(&mut create_session_buffer[..MIN_HEADER_SIZE])
            .unwrap();

        let len2 = self
            .buffer_create_session(
                self.mtu - core::mem::size_of::<usize>() as u16,
                &mut create_session_buffer[MIN_HEADER_SIZE..],
            )
            .unwrap();

        let len = len1 + len2;
        self.wait_session_status(
            &mut create_session_buffer[..len],
            MAX_SESSION_CONNECTION_ATTEMPTS,
        )
    }

    fn buffer_create_session(&self, mtu: u16, buf: &mut [u8]) -> error::Result<usize> {
        let payload = CREATE_CLIENT_Payload(CLIENT_Representation {
            xrce_cookie: [b'X', b'R', b'C', b'E'],
            xrce_version: [0x01u8, 0x00u8],
            xrce_vendor_id: [0x01, 0x0F],
            client_key: self.info.key,
            session_id: self.info.id,

            #[cfg(all(
                not(feature = "hard-liveliness-check"),
                not(feature = "profile-shared-memory")
            ))]
            properties: None,

            #[cfg(all(
                not(feature = "hard-liveliness-check"),
                feature = "profile-shared-memory"
            ))]
            properties: Some([Property {
                name: "uxr_sm",
                value: "1",
            }]),

            #[cfg(all(
                feature = "hard-liveliness-check",
                not(feature = "profile-shared-memory")
            ))]
            properties: Some([Property {
                name: "uxr_hl",
                value: "999999",
            }]),

            #[cfg(all(feature = "hard-liveliness-check", feature = "profile-shared-memory"))]
            properties: Some([
                Property {
                    name: "uxr_sm",
                    value: "1",
                },
                Property {
                    name: "uxr_hl",
                    value: "999999",
                },
            ]),

            mtu,
        });

        payload.to_slice(buf)
    }

    fn wait_session_status(&mut self, buf: &[u8], attempts: usize) -> SessionResult<()> {
        if attempts == 0 {
            self.transport.send_msg(buf).unwrap();
            return Ok(());
        }

        for _ in 0..attempts {
            self.transport.send_msg(buf).unwrap();

            let start_timestamp = self.transport.now();
            let remaining_time = MIN_SESSION_CONNECTION_INTERVAL;
        }

        Ok(())
    }

    fn read_message(&self, buf: &[u8]) -> SessionResult<()> {
        let header = MessageHeader::from_slice(buf).map_err(|_| Error::InvalidData)?;

        let mut correct_msg: bool = false;
        if header.session_id == self.info.id {
            if SESSION_ID_WITHOUT_CLIENT_KEY > self.info.id {
                if let Some(key) = header.key {
                    if key == self.info.key {
                        correct_msg = true;
                    } else {
                        correct_msg = false;
                    }
                } else {
                    correct_msg = false;
                }
            } else {
                correct_msg = true;
            }
        };

        if correct_msg {
            let id = StreamId::from_raw(
                header.session_id,
                crate::stream_id::StreamDirection::InputStream,
            );
            if header.key.is_some() {
                self.read_stream(&buf[(MAX_HEADER_SIZE - 1)..], id, header.sequence_num);
            } else {
                self.read_stream(&buf[(MIN_HEADER_SIZE - 1)..], id, header.sequence_num);
            }
            Ok(())
        } else {
            Err(Error::InvalidData)
        }
    }

    fn read_stream(&self, buf: &[u8], stream_id: StreamId, seq_num: u16) {
        match stream_id.type_u {
            stream_id::StreamType::NoneStream => {}
            stream_id::StreamType::BestEffortStream => {}
            stream_id::StreamType::ReliableStream => {}
            stream_id::StreamType::SharedMemoryStream => {
                unimplemented!()
            }
        };
    }

    fn read_submessage_list(buf: &[u8], stream_id: StreamId) -> Result<()> {
        let mut pos = 0;
        while let Ok(submessage_hdr) = submessage::SubMessageHeader::from_slice(&buf[pos..]) {
            match submessage_hdr {
                SubMessageHeader::Data(len, format) => {

                }
                _ => { unimplemented!() }
            }
        }
        Ok(())
    }

    fn listen_message(&'storage mut self, remaining_time: i32) -> Result<()> {
        let recv = self.transport.receive_msg(remaining_time)?;

        if recv.len() != 0 {
            // self.
            // let header = MessageHeader::from_slice(recv).map_err(|_| Error::InvalidData)?;
            // if header.session_id == self.info.id {
            //     if let Some(key) = header.key {
            //         if key == self.info.key {
            //             let id = StreamId::from_raw(
            //                 header.session_id,
            //                 StreamDirection::InputStream,
            //             );
            //             match id.type_u {
            //                 StreamType::NoneStream => {
            //                     let stream_id = StreamId::from_raw(
            //                         0x00, StreamDirection::InputStream);
            //                     if header.key.is_some() {
            //                         Self::read_submessage_list(&recv[(MAX_HEADER_SIZE - 1)..], stream_id)?;
            //                     } else {
            //                         Self::read_submessage_list(&recv[(MIN_HEADER_SIZE - 1)..], stream_id)?;
            //                     }
            //                 },
            //                 StreamType::BestEffortStream => {
            //                     unimplemented!()
            //                 },
            //                 StreamType::ReliableStream => {
            //                     unimplemented!()
            //                 },
            //                 StreamType::SharedMemoryStream => {
            //                     unimplemented!()
            //                 }
            //             };
            //         } else {
            //             return Err(Error::InvalidData);
            //         }
            //     }
            // } else {
            //     return Err(Error::InvalidData);
            // }
        }
        Ok(())
    }
}
