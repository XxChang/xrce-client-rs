#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

// use panic_rtt_target as _;

use cortex_m_rt::entry;
use serde::Serializer;
use stm32f1xx_hal as _ ;
// use stm32f1xx_hal::{
//     gpio::gpioa::PA4,
//     gpio::gpioa::PA8,
//     gpio::gpioc::PC13,
//     gpio::{Output, PushPull, Input, PullUp},
//     pac::{Peripherals, SPI1},
//     prelude::*,
//     spi::{Pins, Spi, Spi1NoRemap},
//     timer::SysDelay,
// };

// use xrce_client_rs::micro_cdr;

// use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    // rtt_init_print!() ;

    // let mut buf = [0u8;256] ;

    // let mut writer = micro_cdr::Encoder::new(&mut buf) ;
    
    // let v = -32700i16;
    
    // writer.serialize_i16(v).unwrap() ;

    // rprintln!("{:?}", buf) ;
    loop {
        
    }
}
