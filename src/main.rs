//! Watch code for gary600's GPS watch

#![no_std]
#![no_main] // bootup is handled by cortex-m-rt and rtic


// Load panic handler
use panic_semihosting as _;

mod logging;
mod error;
mod state;
mod peripherals;

use crate::logging::SemihostingLogger;

/// Global logger
static LOGGER: SemihostingLogger = SemihostingLogger::new(log::Level::Info);

// RTIC app: handles concurrency and interrupts
#[rtic::app(device = stm32l0xx_hal::pac, peripherals = true)]
mod app {

    // Imports
    use stm32l0xx_hal::{
        self as hal,
        prelude::*
    };

    #[shared]
    struct Shared {

    }

    #[local]
    struct Local {

    }


    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize logging
        // Must use `set_logger_racy` as normal `set_logger` doesn't work on thumbv6.
        // This is safe because this is run before anything else and it's the only initialization
        // Fail silently
        let _ = unsafe { log::set_logger_racy(&crate::LOGGER) };
        log::trace!("logging initialized");

        // Peripheral shorthand
        let dp: hal::pac::Peripherals = ctx.device;
        //let cp: hal::pac::CorePeripherals = ctx.core;

        // Configure clock: High Speed Internal @ 16 MHz
        log::trace!("setting up RCC");
        let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

        // Configure power
        log::trace!("setting up PWR");
        let mut pwr = hal::pwr::PWR::new(dp.PWR, &mut rcc);

        // Configure RTC (enables Low Speed External oscillator @ 32768 Hz)
        log::trace!("setting up RTC");
        let mut rtc = hal::rtc::Rtc::new(dp.RTC, &mut rcc, &mut pwr, None).unwrap();
        log::trace!("enabling wakeup interrupt");
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
        //let gpiob = dp.GPIOB.split(&mut rcc);

        // Buzzer PWM
        log::trace!("creating buzzer");
        let mut pwm_timer = hal::pwm::Timer::new(dp.TIM2, 2000.Hz(), &mut rcc);
        let buzzer = crate::peripherals::alert::Buzzer::new(pwm_timer.channel1, gpioa.pa0);

        // Vibrate motor
        let vibrate_pin = gpioa.pa1.into_push_pull_output();

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
        //let sdcard_cs = gpioa.pa3.into_push_pull_output();

        // Create display and clear
        log::trace!("setting up display");
        let mut display = crate::peripherals::display::SharpLcd::new(spi, display_cs);
        display.send_clear().unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }
}

/// Default interrupt handler: log the interrupt
#[cortex_m_rt::exception]
#[allow(non_snake_case)]
fn DefaultHandler(irqn: i16) {
    log::trace!("Unhandled interrupt: {}", irqn);
}