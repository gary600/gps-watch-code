//! Logging utils

use core::fmt::Write;
use log::{Log, Record, Level, Metadata};
use cortex_m_semihosting::hio::hstderr;

/// An implementation of [`log::Log`] for printing to the host PC
pub struct SemihostingLogger {
    level: Level
}

impl Log for SemihostingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }


    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut fd = match hstderr() {
                Ok(fd) => fd,
                Err(()) => return // Fail silently if unable to get hstderr
            };

            // Write line header
            write!(
                fd,
                "[{level} {target}] {file}:{line}: ",
                level=record.level(),
                target=record.target(),
                file=record.file().unwrap_or("<unknown>"),
                line=record.line().unwrap_or(0)
            ).unwrap();

            // Write the formatted output
            fd.write_fmt(*record.args()).unwrap();
        }
    }

    fn flush(&self) { } // Do nothing: semihosting macros are unbuffered
}

impl SemihostingLogger {
    pub const fn new(level: Level) -> Self {
        Self {
            level
        }
    }
}