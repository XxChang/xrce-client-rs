#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
};
use xrce_client_rs::session::Session;
use xrce_client_rs::serial_transport::SerialTransport;
use xrce_client_rs::time::Clock;

use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};

struct Test ;

impl Clock for Test {
    fn now(&mut self) -> i64 {
        0
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Get access to the device specific peripherals from the peripheral access crate
    let p = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = p.AFIO.constrain();

    // Prepare the GPIOB peripheral
    let mut gpioa = p.GPIOA.split();

    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    // Set up the usart device. Take ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    let serial = Serial::new(
        p.USART1,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(115200.bps()),
        &clocks,
    );

    let (tx, rx) = serial.split();
    
    let uart = SerialTransport::new(tx, rx);

    let mut session = Session::new([0xAA, 0xAA, 0xBB, 0xBB], uart, Test{});

    session.create().unwrap();
    loop {
        
    }
}