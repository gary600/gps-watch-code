//! Logging utils

use core::fmt::Write;
use log::{Log, Record, Level, Metadata};
use cortex_m_semihosting::hio::hstdout;

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
            let mut fd = match hstdout() {
                Ok(fd) => fd,
                Err(()) => return // Fail silently if unable to get hstdout
            };

            // Write line header, fail silently
            let _ = write!(
                fd,
                "[{level} {target}] {file}:{line}: ",
                level=record.level(),
                target=record.target(),
                file=record.file().unwrap_or("<unknown>"),
                line=record.line().unwrap_or(0)
            );

            // Write the formatted output, fail silently
            let _ = fd.write_fmt(*record.args());
        }
    }

    fn flush(&self) { } // Do nothing: semihosting ops are unbuffered
}

impl SemihostingLogger {
    pub const fn new(level: Level) -> Self {
        Self {
            level
        }
    }
}