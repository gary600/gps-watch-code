//! Contains an implementation of [`DrawTarget`](embedded_graphics::draw_target::DrawTarget) for the Sharp Memory LCD.
//!
//! Most implementation details are based on [this programming guide](https://www.sharpsde.com/fileadmin/products/Displays/2016_SDE_App_Note_for_Memory_LCD_programming_V1.3.pdf).

use core::fmt::{Formatter};
use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    pixelcolor::BinaryColor
};
use embedded_hal::{
    digital::v2::OutputPin,
    spi::FullDuplex
};

const WIDTH: usize = 168;
const HEIGHT: usize = 144;

const COMMAND_WRITE_LINES: u8 = 0b10000000;
const COMMAND_CLEAR: u8 = 0b00100000;
const COMMAND_TOGGLE_VCOM: u8 = 0b00000000;

/// An implementation of [`DrawTarget`](embedded_graphics::draw_target::DrawTarget) for the Sharp Memory LCD
pub struct SharpLcd<SPI, CS> {
    spi: SPI,
    cs: CS,

    /// Contains an array for every row, and a `u8` for every 8 pixels in that row
    /// Byte order: leftmost pixels are in the LSB (this is how the LCD expects it)
    framebuffer: [[u8; WIDTH/8]; HEIGHT],
    /// Keeps track of which lines have been updated. LSB means lower y-value
    updated_lines: [u8; HEIGHT/8],
    vcom: bool
}
impl<SPI, CS> core::fmt::Debug for SharpLcd<SPI, CS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SharpLcd")
            .field("framebuffer", &self.framebuffer)
            .field("updated_lines", &self.updated_lines)
            .field("vcom", &self.vcom)
            .finish_non_exhaustive() // because the SPI and CS don't have Debug
    }
}

impl<SPI: FullDuplex<u8>, CS: OutputPin> SharpLcd<SPI, CS> {
    /// Create a new display
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self {
            spi,
            cs,
            framebuffer: [[0; WIDTH/8]; HEIGHT],
            updated_lines: [0; HEIGHT/8],
            vcom: false
        }
    }

    /// Clear the screen fully white by sending a command to the screen.
    pub fn send_clear(&mut self) -> nb::Result<(), SPI::Error> {
        self.framebuffer = [[0; WIDTH/8]; HEIGHT];

        // Assert CS
        let _ = self.cs.set_high(); // cannot error
        // Header
        self.spi.send(self.format_command(COMMAND_CLEAR))?;
        // Trailer
        self.spi.send(0x00)?;
        // Un-assert CS
        let _ = self.cs.set_low();

        // Lines have been flushed
        self.updated_lines = [0x00; HEIGHT/8];

        Ok(())
    }

    /// Flush changes to the screen
    pub fn flush(&mut self) -> nb::Result<(), SPI::Error> {
        //TODO: Might want to optimize with DMA, that'd require more params

        // Assert CS
        let _ = self.cs.set_high();

        // If no lines have been updated since last flush, then just write Toggle VCOM message
        if self.updated_lines == [0x00; HEIGHT/8] {
            // Header
            self.spi.send(self.format_command(COMMAND_TOGGLE_VCOM))?;
            // Trailer
            self.spi.send(0x00)?;
        }

        // If framebuffer has been updated, only send changed lines
        else {
            // Header
            self.spi.send(self.format_command(COMMAND_WRITE_LINES))?;

            // Line data
            for (n, line) in self.framebuffer.iter().enumerate() {
                // Only send line if changed
                if self.updated_lines[n / 8] & (1u8 << (n % 8)) != 0 {
                    // Line number
                    self.spi.send(n as u8)?;
                    // Line data
                    for &byte in line {
                        self.spi.send(byte)?;
                    }
                    // Line trailer
                    self.spi.send(0x00)?;
                }
            }

            // Trailer
            self.spi.send(0x00)?;
            // Un-assert CS
            let _ = self.cs.set_low();

            // Clear changed lines
            self.updated_lines = [0x00; HEIGHT / 8];
        }

        Ok(())
    }
}

impl<SPI, CS> SharpLcd<SPI, CS> {
    /// Clear the screen fully white. Needs to be flushed afterward.
    pub fn clear(&mut self) {
        // Clear framebuffer
        self.framebuffer = [[0; WIDTH/8]; HEIGHT];
        // All lines have been updated
        self.updated_lines = [0xFF; HEIGHT/8];
    }

    /// Toggle the VCOM value. This should be done at least once per second to prevent burn-in.
    /// [`SharpLcd::flush()`] should be called afterwards.
    pub fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
    }

    /// Write a single pixel.
    #[inline(always)]
    fn write_pixel(&mut self, pixel: Pixel<BinaryColor>) {
        // Ignore it if it's outside of the screen bounds
        if !self.bounding_box().contains(pixel.0) {
            return;
        }

        // Write to framebuffer. Shouldn't panic because bounds have been checked
        if pixel.1.is_on() {
            // Or'd with bit to set it
            self.framebuffer[pixel.0.y as usize][(pixel.0.x / 8) as usize]
                |= 1u8 << (pixel.0.x % 8);
        }
        else {
            // And'd with inverse of byte to unset it
            self.framebuffer[pixel.0.y as usize][(pixel.0.x / 8) as usize]
                &= !(1u8 << (pixel.0.x % 8));
        }

        // Mark line as updated
        self.updated_lines[(pixel.0.y / 8) as usize] |= 1u8 << (pixel.0.y % 8);
    }

    /// Format a display command with the V-bit set to the VCOM state
    #[inline(always)]
    fn format_command(&self, command: u8) -> u8 {
        command | ((self.vcom as u8) << 6)
    }
}

impl<SPI, CS> Dimensions for SharpLcd<SPI, CS> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(WIDTH as u32, HEIGHT as u32))
    }
}

impl<SPI, CS> DrawTarget for SharpLcd<SPI, CS> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        // Draw each pixel
        for pixel in pixels {
            self.write_pixel(pixel);
        }

        Ok(())
    }
}