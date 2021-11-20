//! Watch code for gary600's GPS watch

#![no_std]
#![no_main] // while there is a main() function, it's called manually by rt::boot()

use core::fmt::Write;
use stm32l0xx_hal::{
    self as hal,
    prelude::*
};
use embedded_graphics::{
    self as gfx,
    prelude::*
};
use chrono::prelude::*;

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
    log::trace!("taking peripherals");
    let dp = hal::pac::Peripherals::take().unwrap();
    let mut cp = hal::pac::CorePeripherals::take().unwrap();

    // Configure clock: 16 MHz internal
    log::trace!("setting up RCC");
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

    // Configure power
    log::trace!("setting up PWR");
    let mut pwr = hal::pwr::PWR::new(dp.PWR, &mut rcc);

    // Configure RTC
    log::trace!("setting up RTC");
    let mut rtc = hal::rtc::Rtc::new(dp.RTC, &mut rcc, &mut pwr, None).unwrap();
    // Enable wakeup timer interrupt
    rtc.enable_interrupts(hal::rtc::Interrupts {
        timestamp: false,
        wakeup_timer: true,
        alarm_a: false,
        alarm_b: false
    });
    // Set 1 second wakeup timer
    rtc.wakeup_timer().start(32768u32); //TODO: Docs are unclear on RTC frequency

    // Acquire GPIO for pins
    log::trace!("acquiring GPIO");
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Create SPI for the display
    log::trace!("setting up SPI1");
    let sck = gpioa.pa5;
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7;
    let spi = dp.SPI1.spi(
        (sck, miso, mosi),
        hal::spi::MODE_0, // CS is idle-low, Clock is capture-on-rise as per LCD guide
        2_000_000.Hz(), // 2 Mhz clock as suggested in LCD guide
        &mut rcc
    );

    let display_cs = gpioa.pa4.into_push_pull_output();
    let sdcard_cs = gpioa.pa3.into_push_pull_output();

    // Create display and clear
    log::trace!("setting up display");
    let mut display = crate::display::SharpLcd::new(spi, display_cs);
    display.clear();
    display.flush();

    // Create sdcard
    //let sdcard = embedded_sdmmc::SdMmcSpi::new(spi, sdcard_cs);

    // Drawing stuff
    let text_style = gfx::mono_font::MonoTextStyle::new(
        &gfx::mono_font::iso_8859_1::FONT_4X6,
        gfx::pixelcolor::BinaryColor::On
    );

    // MAIN LOOP
    log::trace!("entering main loop");
    loop {
        log::trace!("top of main loop");

        // Draw time
        let time = rtc.now().time();
        // Create buffer for time string (should fit in 8 chars)
        let mut time_str = arrayvec::ArrayString::<8>::new();
        // Format time string
        write!(time_str, "{:02}:{:02}:{:02}", time.hour(), time.minute(), time.second()).unwrap();
        // Draw text to framebuffer
        gfx::text::Text::new(&time_str, Point::new(1, 1), text_style).draw(&mut display);
        // Toggle VCOM to prevent charge buildup in screen
        display.toggle_vcom();
        // Write framebuffer to screen
        display.flush().unwrap();


        // Enter low-power sleep mode until the RTC interrupt fires
        log::trace!("entering low-power sleep mode");
        pwr.low_power_sleep_mode(&mut cp.SCB, &mut rcc).enter();
    }
}