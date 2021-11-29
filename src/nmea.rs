//! NMEA 0183 parser using fixed-point arithmetic.

use arrayvec::ArrayVec;
use chrono::NaiveTime;

/// NMEA 0183 Parser
#[derive(Debug)]
pub struct NmeaParser {
    buf: ArrayVec<u8, 79>
}

impl NmeaParser {
    pub fn new() -> Self {
        Self {
            buf: ArrayVec::new()
        }
    }

    pub fn parse_from_byte(&mut self, byte: u8) -> Option<Result<NmeaSentence, NmeaError>> {
    todo!()
    }
}

/// NMEA 0183 parse error
#[derive(Debug)]
pub enum NmeaError {
    /// Unexpected character
    UnexpectedCharacter,

}

pub struct Coord {
    hemisphere: bool, // pos = true
    degrees: u8, // 0-90 or 0-180
    minutes: u8, // 0-60
    frac_minutes: u16
}

pub enum FixType {
    Invalid = 0,
    Autonomous = 1,
    Dgps = 2,
    Pps = 3,
    Rtk = 4,
    RtkFloat = 5,
    Estimated = 6,
    Manual = 7,
    Simulation = 8,
    Waas = 9
}

/// NMEA 0183 resulting sentence
#[derive(Debug, Copy, Clone)]
pub enum NmeaSentence {
    /// Fix Data
    Gga {
        time: NaiveTime,
        latitude: Coord,
        longitude: Coord,
        fix_type: FixType,
        satellites: u8,
        hdop: u8
    },
    /// Geographic Position
    Gll {
        latitude: Coord,
        longitude: Coord,
        time: NaiveTime,

    },
    /// Dilution of Precision and Satellites
    Gsa {

    }
}
}