//! Watch code for gary600's GPS watch

#![no_std]
#![no_main]

use stm32l0xx_hal::{
    self as hal,
    prelude::*
};

mod rt;
mod logging;
mod error;
mod state;
mod display;

use crate::logging::SemihostingLogger;
use crate::error::MainError;


/// Global logger (initialized by [`rt::boot()`](rt))
static LOGGER: SemihostingLogger = SemihostingLogger::new(log::Level::Info);


/// The "real" main function. Wrapped by [`rt::boot()`](rt) to allow it to return errors.
fn main() -> Result<(), MainError> {
    log::trace!("entered main()");

    // Grab peripherals
    // Will not panic because this is the only time take() is called
    let dp = hal::pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Configure clock: 16 MHz internal
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

    // Acquire GPIOA
    let gpioa = dp.GPIOA.split(&mut rcc);

    // Create SPI for the display
    let cs = gpioa.pa4.into_push_pull_output();
    let sck = gpioa.pa5;
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7;
    let mut spi = dp.SPI1.spi(
        (sck, miso, mosi),
        hal::spi::MODE_0, // CS is idle-low, Clock is capture-on-rise as per LCD guide
        2_000_000.Hz(), // 2 Mhz clock as suggested in LCD guide
        &mut rcc
    );

    // Create display and clear
    let mut display = crate::display::SharpLcd::new(spi, cs);
    display.clear();
    display.flush();

    loop {}
}