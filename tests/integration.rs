#![no_std]
#![no_main]

use defmt_rtt as _;
use stm32f1xx_hal as _;
use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

#[cfg(test)]
#[defmt_test::tests]
mod tests {

    #[test]
    fn it_works() {
        assert!(true)
    }
}
