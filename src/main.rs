//! Watch code for gary600's GPS watch

#![no_std]
#![no_main] // while there is a main() function, it's called manually by rt::boot()

use stm32l0xx_hal::{
    self as hal,
    prelude::*
};

mod rt;
mod logging;
mod error;
mod state;
mod peripherals;

use crate::logging::SemihostingLogger;
use crate::error::MainError;
use crate::state::{Resources, SharedState, State};


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

    // Enable low-speed external oscillator (for RTC)
    log::trace!("setting up LSE");
    let lse = rcc.enable_lse(&pwr);

    // Configure RTC
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
    log::trace!("creating buzzer timer");
    let buzzer_timer = hal::pwm::Timer::new(dp.TIM2, 2000.Hz(), &mut rcc);
    buzzer_timer.channel1.assign(gpioa.pa0);
    let buzzer = peripherals::alert::Buzzer::new(buzzer_timer);

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
    let mut display = peripherals::display::SharpLcd::new(spi, display_cs);
    display.send_clear().unwrap();

    // Create sdcard
    //let sdcard = embedded_sdmmc::SdMmcSpi::new(spi, sdcard_cs);

    // Initalize state machine
    let shared_state = SharedState::new();

    let resources = Resources {
        rtc,
        display,
        buzzer
    };

    let mut state = State::new(resources, shared_state);

    // MAIN LOOP
    log::trace!("entering main loop");
    loop {
        log::trace!("top of main loop");

        // Update state
        log::trace!("updating state");
        state.update();

        // Redraw display if needed
        log::trace!("redrawing display");
        state.draw();


        // Enter low-power sleep state until the RTC interrupt fires
        log::trace!("entering low-power sleep state");
        pwr.low_power_sleep_mode(&mut cp.SCB, &mut rcc).enter();
    }
}