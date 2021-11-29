//! Watch code for gary600's GPS watch
//! [MCU datasheet](https://www.st.com/resource/en/datasheet/stm32l071kz.pdf)
//! [MCU programming manual](https://www.st.com/resource/en/programming_manual/pm0223-cortexm0-programming-manual-for-stm32l0-stm32g0-stm32wl-and-stm32wb-series-stmicroelectronics.pdf)

#![no_std]
#![no_main] // bootup is handled by cortex-m-rt and rtic


// Load panic handler
use panic_semihosting as _;

mod logging;
mod error;
mod state;
mod peripherals;
mod nmea;

use crate::logging::SemihostingLogger;

/// Global logger
static LOGGER: SemihostingLogger = SemihostingLogger::new(log::Level::Info);

// RTIC app: handles concurrency and interrupts
#[rtic::app(
    device = stm32l0xx_hal::pac, // The device's peripheral access crate
    peripherals = true, // Whether or not RTIC should grab the device's peripherals
    dispatchers = [LPTIM1] // Interrupts that aren't otherwise used, for triggering software tasks
)]
mod app {
    // Imports
    use stm32l0xx_hal::{
        self as hal,
        prelude::*,
        pac::SPI1,
        rcc::Rcc,
        pwr::PWR,
        rtc::Rtc,
        spi::Spi
    };
    use crate::peripherals as perif;

    // Resource types
    #[shared]
    struct Shared {
        rcc: Rcc,
        pwr: PWR,
        rtc: Rtc,
        // haha yes i love type signatures
        display: perif::SharpLcd<
            Spi<
                SPI1,
                (
                    hal::gpio::gpioa::PA5<hal::gpio::Analog>,
                    hal::gpio::gpioa::PA6<hal::gpio::Analog>,
                    hal::gpio::gpioa::PA7<hal::gpio::Analog>
                )
            >,
            hal::gpio::gpioa::PA4<hal::gpio::Output<hal::gpio::PushPull>>
        >,
        gps: perif::Gps
    }

    #[local]
    struct Local {

    }

    // Monotonics
    #[monotonic(binds = SysTick, default = true)]
    type SystickMonotonic = systick_monotonic::Systick<100>; // General timer for event scheduling, 10 ms precision


    // Initalization function. Called on bootup after RTIC is initialized, to setup shared resources
    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize logging
        // Must use `set_logger_racy` as normal `set_logger` doesn't work on thumbv6.
        // This is safe because this is run before anything else and it's the only initialization
        // Fail silently
        let _ = unsafe { log::set_logger_racy(&crate::LOGGER) };
        log::trace!("logging initialized");

        // Peripheral shorthand
        let dp: hal::pac::Peripherals = c.device;
        let cp: cortex_m::Peripherals = c.core;

        // Configure clock: High Speed Internal @ 16 MHz
        log::trace!("setting up RCC");
        let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

        // Configure power
        log::trace!("setting up PWR");
        let mut pwr = PWR::new(dp.PWR, &mut rcc);

        // Enable Low Speed External oscillator @ 32768 Hz
        // Used by RTC and LPUART
        // Safe to enable multiple times
        log::trace!("enabling LSE");
        let _lse = rcc.enable_lse(&pwr);

        // Configure RTC
        log::trace!("setting up RTC");
        let mut rtc = Rtc::new(dp.RTC, &mut rcc, &mut pwr, None).unwrap();
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
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Buzzer PWM
        log::trace!("creating buzzer");
        let pwm_timer = hal::pwm::Timer::new(dp.TIM2, 2000.Hz(), &mut rcc);
        let _buzzer = perif::Buzzer::new(pwm_timer.channel1, gpioa.pa0);

        // Vibrate motor
        let mut vibrate_pin = gpioa.pa1.into_push_pull_output();
        let _ = vibrate_pin.set_low();

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
        let mut display = perif::SharpLcd::new(spi, display_cs);
        display.send_clear().unwrap();

        // Create UART for GPS
        log::trace!("setting up LPUART");
        let mut gps_uart = dp.LPUART1.usart(
            // gpioc.pc4,
            // gpioc.pc5,
            gpioc.pc1,
            gpioc.pc0,
            hal::serial::Config::default().baudrate(4800.Bd()), // NMEA 0183 uses 4800 by default
            &mut rcc
        ).unwrap();
        log::trace!("creating GPS object");
        let gps = perif::Gps::new(gps_uart);

        // Create Systick monotonic object
        log::trace!("creating systick monotonic");
        let syst = systick_monotonic::Systick::new(cp.SYST, rcc.clocks.sys_clk().0);

        log::info!("initalization complete");
        (
            Shared {
                rcc,
                pwr,
                rtc,
                display,
                gps
            },
            Local {},
            init::Monotonics(syst)
        )
    }

    /// Triggers on RTC
    #[task(binds = RTC, shared = [rtc])]
    fn on_rtc(_c: on_rtc::Context) {
        log::trace!("on_rtc()");

        todo!()
    }

    /// Triggers on LPUART
    #[task(binds = AES_RNG_LPUART1, shared = [gps])] // Weird interrupt name because it's shared by AES and LPUART?
    fn on_lpuart(mut c: on_lpuart::Context) {
        log::trace!("on_lpuart()");

        // Acquire lock on gps resource
        c.shared.gps.lock(|gps: &mut perif::Gps| {
            // Parse all received bytes into sentences
            let _sentences = gps.recv();

            todo!();
        });
    }

    /// Toggles VCOM and flushes the display's buffer every second
    #[task(shared = [display])]
    fn flush_display(mut c: flush_display::Context) {
        log::trace!("flush_display()");

        // Acquire lock on display resource
        c.shared.display.lock(|disp: &mut perif::SharpLcd<_, _>| {
            // Toggle VCOM as required by display spec
            disp.toggle_vcom();
            // Flush display, print any SPI errors
            if let Err(e) = disp.flush() {
                log::error!("error flushing display: {:?}", e);
            }
        });
    }

    /// Updates the state
    #[task]
    fn update(_c: update::Context) {
        log::trace!("update()");

        todo!()
    }
}