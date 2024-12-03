#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;
use serde::de::Deserialize;
use serde::ser::Serialize;
use stm32f1xx_hal as _;
use xrce_client_rs::micro_cdr;

#[entry]
fn main() -> ! {
    let mut buf = [0u8; 25];

    let mut encoder = micro_cdr::Encoder::new(&mut buf);

    Serialize::serialize("hello world!", &mut encoder).unwrap();

    let mut decoder = micro_cdr::Decoder::new(&buf);

    let s: &str = Deserialize::deserialize(&mut decoder).unwrap();

    defmt::info!("{:?}", s);

    loop {}
}
