#![allow(clippy::empty_loop)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::serial;
use nb::block;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    serial::{Config, Serial},
    timer::CounterMs,
};
use xrce_client_rs::serial::transport::SerialTransport;
use xrce_client_rs::serial::SerialPlatformOps;
use xrce_client_rs::session::Session;

struct SerialInterface<TX, RX> {
    tx: TX,
    rx: RX,
    clock: CounterMs<pac::TIM1>,
}

impl<TX, RX> SerialPlatformOps for SerialInterface<TX, RX>
where
    TX: serial::Write<u8>,
    RX: serial::Read<u8>,
{
    fn read_serial_data(
        &mut self,
        buf: &mut [u8],
        len: usize,
        timeout: i32,
    ) -> xrce_client_rs::Result<usize> {
        let mut timeout = timeout;
        let mut ready_data: usize = 0;

        loop {
            let e = self.rx.read();
            match e {
                Err(nb::Error::Other(_)) => {}
                Err(nb::Error::WouldBlock) => {}
                Ok(x) => {
                    buf[ready_data] = x;
                    ready_data += 1;
                }
            };

            if ready_data == len {
                return Ok(ready_data);
            }

            timeout -= self.millis();
            if timeout < 0 && ready_data == 0 {
                return Err(xrce_client_rs::Error::Timeout);
            } else if timeout < 0 && ready_data > 0 {
                return Ok(ready_data);
            }
        }
    }

    fn write_serial_data(&mut self, buf: &[u8]) -> xrce_client_rs::Result<usize> {
        let mut bytes_written: usize = 0;
        for byte in buf {
            block!(self.tx.write(*byte)).ok();
            bytes_written += 1;
        }

        Ok(bytes_written)
    }

    #[inline]
    fn millis(&mut self) -> i32 {
        self.clock.now().ticks().try_into().unwrap()
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
    let mut timer = p.TIM1.counter_ms(&clocks);
    timer.start(1.secs()).unwrap();

    let uart = SerialInterface {
        tx,
        rx,
        clock: timer,
    };

    let transport = SerialTransport::new(uart, 0, 1);
    let mut session = Session::new([0xAA, 0xAA, 0xBB, 0xBB], transport);

    session.create().unwrap();
    loop {}
}
