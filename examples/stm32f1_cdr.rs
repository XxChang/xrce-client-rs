#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f1xx_hal as _ ;
use xrce_client_rs::micro_cdr;
use serde::ser::Serialize;
use serde::de::Deserialize;

use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut buf = [0u8;25];

    let mut encoder = micro_cdr::Encoder::new(&mut buf) ;
    
    Serialize::serialize("hello world!", &mut encoder).unwrap();

    let mut decoder = micro_cdr::Decoder::new(&mut buf) ;

    let s: &str = Deserialize::deserialize(&mut decoder).unwrap();
    
    rprintln!("{:?}", s) ;

    loop {
        
    }
}
