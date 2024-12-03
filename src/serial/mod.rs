use embedded_hal::delay::DelayNs;

use crate::Result;

pub mod transport;

pub trait SerialPlatformOps {
    fn write_serial_data(&mut self, buf: &[u8]) -> Result<usize>;

    fn read_serial_data(&mut self, buf: &mut [u8], len: usize, timeout: i32) -> Result<usize>;

    fn millis(&mut self) -> i32;
}
