#![no_std]
#![no_main]

pub mod error;
pub mod micro_cdr;
pub mod session;

mod header;
mod submessage;
mod types;
pub mod serial_transport;

mod communication;
mod lowlevel_io;

pub mod time;

const MIN_SESSION_CONNECTION_INTERVAL: i64 = 1000;
const MAX_SESSION_CONNECTION_ATTEMPTS: usize = 10;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    BigEndianness,
    LittleEndianness,
}

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

    use crate::{header, types::{CREATE_CLIENT_Payload, CLIENT_Representation}};

    #[test]
    fn serialize_create_client() {
        let mut create_session_buffer = [0u8;28];
        header::MessageHeader::new(0x81 & 0x80, 0, 0, None).to_slice(&mut create_session_buffer[..4]).unwrap();

        let payload = CREATE_CLIENT_Payload {
            client_representation: CLIENT_Representation {
                xrce_cookie: [b'X', b'R', b'C', b'E'],
                xrce_version: [0x01u8, 0x00u8],
                xrce_vendor_id: [0x0F, 0x0F],
                client_key: [0x22, 0x33, 0x44, 0x55],
                session_id: 0xDD,
                properties: None,
                mtu: 252,
            }
        } ;

        payload.to_slice(&mut create_session_buffer[4..]).unwrap();
        assert_eq!(
            [   
                // Message Header
                0x80, 
                0x00, 
                0x00, 0x00, 
                // Submessage Header
                0x00, 
                0x01, 
                0x10, 0x00,
                // Payload
                b'X', b'R', b'C', b'E',
                0x01, 0x00,
                0x0F, 0x0F,
                0x22, 0x33, 0x44, 0x55,
                0xDD,
                0x00
            ],
            create_session_buffer[..22],
        );
    }

}
