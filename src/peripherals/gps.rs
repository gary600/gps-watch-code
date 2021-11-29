//! GPS wrapper
use nb::Error as NbError;
use stm32l0xx_hal::{
    prelude::*,
    serial::{
        Serial,
        Event as SerialEvent
    },
    pac::{
        LPUART1
    }
};
use nmea0183::{
    Parser,
    ParseResult
};
use arrayvec::ArrayVec;

pub struct Gps {
    uart: Serial<LPUART1>,
    parser: Parser
}

impl Gps {
    pub fn new(mut uart: Serial<LPUART1>) -> Self {
        // Only send interrupts for Rx events
        uart.listen(SerialEvent::Rxne);
        uart.unlisten(SerialEvent::Txe);
        uart.unlisten(SerialEvent::Idle);

        Self {
            uart,
            parser: Parser::new()
        }
    }

    /// Reads data from the GPS serial.
    pub fn recv(&mut self) -> ArrayVec<ParseResult, 8> {
        // todo!();
        let mut sentences = ArrayVec::new();

        loop {
            // Try to read byte
            let b = match self.uart.read() {
                // If there's a byte to read, get it
                Ok(b) => b,
                Err(e) => match e {
                    // If there's a UART error, log it and continue
                    NbError::Other(e) => {
                        log::error!("UART error: {:?}", e);
                        continue
                    },
                    // If there's nothing left to read, done
                    NbError::WouldBlock => break
                }
            };

            // If the parser has gotten enough data to parse a sentence (or error):
            // if let Some(res) = self.parser.parse_from_byte(b) {
            {
                let res: Result<ParseResult, &str> = Err("foo");
                match res {
                    // If parse successful, add to sentence list
                    Ok(s) => {
                        sentences.push(s); // Shouldn't panic because we're returning if full
                        // If buffer is full after pushing, then return it, and print warning about
                        // too many NMEA sentences
                        if sentences.is_full() {
                            log::warn!("NMEA sentence buffer full without finishing parse!");
                            return sentences
                        }
                    },
                    // Else, log parse error and move on
                    Err(e) => log::error!("NMEA parse error: {:?}", e)
                }
            }

        }

        sentences
    }
}