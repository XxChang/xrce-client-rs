#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![no_main]

pub mod error;
pub mod micro_cdr;
pub mod session;

mod header;
mod stream_id;
mod submessage;
mod types;

mod communication;
pub mod serial;

pub mod time;

const MIN_SESSION_CONNECTION_INTERVAL: i64 = 1000;
const MAX_SESSION_CONNECTION_ATTEMPTS: usize = 10;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    BigEndianness,
    LittleEndianness,
}

#[derive(Debug)]
pub enum Error {
    PartWritten(usize),
    RemoteAddrError,
    Timeout,
    IoError,

    Deined,
    InvalidData,
    Incompatible,
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
use panic_probe as _;

#[cfg(test)]
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod test {
    use defmt_rtt as _;
    use stm32f1xx_hal as _;

    use crate::{
        header::{self, MessageHeader},
        micro_cdr, session, submessage,
        types::{CLIENT_Representation, CREATE_CLIENT_Payload},
    };

    #[test]
    fn ser_de_create_client() {
        let mut create_session_buffer = [0u8; 28];
        {
            MessageHeader::new(0x81 & 0x80, 0, 0, None)
                .to_slice(&mut create_session_buffer[..4])
                .unwrap();

            let payload = CREATE_CLIENT_Payload(CLIENT_Representation {
                xrce_cookie: [b'X', b'R', b'C', b'E'],
                xrce_version: [0x01u8, 0x00u8],
                xrce_vendor_id: [0x0F, 0x0F],
                client_key: [0x22, 0x33, 0x44, 0x55],
                session_id: 0xDD,
                properties: None,
                mtu: 252,
            });

            payload.to_slice(&mut create_session_buffer[4..]).unwrap();
            assert_eq!(
                [
                    // Message Header
                    0x80, 0x00, 0x00, 0x00, // Submessage Header
                    0x00, 0x01, 0x10, 0x00, // Payload
                    b'X', b'R', b'C', b'E', 0x01, 0x00, 0x0F, 0x0F, 0x22, 0x33, 0x44, 0x55, 0xDD,
                    0x00
                ],
                create_session_buffer[..22],
            );
        }

        {
            let header = MessageHeader::from_slice(&create_session_buffer).unwrap();
            assert_eq!(header::MessageHeader::new(0x81 & 0x80, 0, 0, None), header);

            let (submessage_header, payload) = if header.key.is_none() {
                (
                    submessage::SubMessageHeader::from_slice(
                        &create_session_buffer[session::MIN_HEADER_SIZE..],
                    )
                    .unwrap(),
                    CREATE_CLIENT_Payload::from_slice(
                        &create_session_buffer[session::MIN_HEADER_SIZE + 4..],
                    )
                    .unwrap(),
                )
            } else {
                (
                    submessage::SubMessageHeader::from_slice(
                        &create_session_buffer[session::MAX_HEADER_SIZE..],
                    )
                    .unwrap(),
                    CREATE_CLIENT_Payload::from_slice(
                        &create_session_buffer[session::MAX_HEADER_SIZE + 4..],
                    )
                    .unwrap(),
                )
            };
            assert_eq!(
                submessage::SubMessageHeader::CreateClient(16),
                submessage_header
            );
            assert_eq!(payload.0.xrce_cookie, [b'X', b'R', b'C', b'E']);
        }
    }

    #[test]
    fn ser_de_submessageheader() {
        let mut submessage_header_buf = [0u8; 256];
        {
            let header = submessage::SubMessageHeader::AckNack(20);
            header.to_slice(&mut submessage_header_buf).unwrap();
        }

        {
            let header = submessage::SubMessageHeader::from_slice(&submessage_header_buf).unwrap();
            assert_eq!(header, submessage::SubMessageHeader::AckNack(20));
        }
    }

    #[test]
    fn align_test() {
        let v: bool = true;
        let mut buf = [0u8; 256];
        {
            let mut writer = micro_cdr::Encoder::new(&mut buf);
            serde::Serializer::serialize_bool(&mut writer, v).unwrap();
            serde::Serializer::serialize_f32(&mut writer, 32.0).unwrap();
            assert_eq!(writer.offset, 8);

            writer.set_pos_of::<f32>().unwrap();
            assert_eq!(writer.offset, 8);
            serde::Serializer::serialize_bool(&mut writer, v).unwrap();
            assert_eq!(writer.offset, 9);

            writer.set_pos_of::<f64>().unwrap();
            assert_eq!(writer.offset, 16);
        }
    }
}
