//! Things that interact with the runtime, such as the true entry point and interrupt handlers

// Load panic handler
use panic_semihosting as _;

/// `boot()`: Main entry point: wraps [`main()`](crate::main)
#[cortex_m_rt::entry]
fn boot() -> ! {
    // Initialize logging
    // Must use `set_logger_racy` as normal `set_logger` doesn't work on thumbv6
    // Since this is the only logger initialization ever, this is safe.
    // Fail silently
    let _ = unsafe { log::set_logger_racy(&crate::LOGGER) };
    log::trace!("logging initialized");

    // Call the real main()
    let e = crate::main();

    panic!("main() returned: {:?}", e);
}

// Exception handlers

/// `HardFault()`: Panic on hard faults (hopefully the panic handler still works!)
#[cortex_m_rt::exception]
#[allow(non_snake_case)] // function name is hard-coded
fn HardFault(exception_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault: {:?}", exception_frame);
}

/// `DefaultHandler()`: Log unknown interrupts
#[cortex_m_rt::exception]
#[allow(non_snake_case)]
fn DefaultHandler(irqn: i16) {
    log::trace!("Unknown interrupt: {}", irqn);
}