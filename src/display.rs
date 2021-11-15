//! Contains an implementation of [`DrawTarget`](embedded_graphics::draw_target::DrawTarget) for the Sharp Memory LCD.
//!
//! Most implementation details are based on [this programming guide](https://www.sharpsde.com/fileadmin/products/Displays/2016_SDE_App_Note_for_Memory_LCD_programming_V1.3.pdf).

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
    vcom: bool
}

impl<SPI: FullDuplex<u8>, CS: OutputPin> SharpLcd<SPI, CS> {
    /// Create a new display
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self {
            spi,
            cs,
            framebuffer: [[0; WIDTH/8]; HEIGHT],
            vcom: false
        }
    }

    /// Clear the screen fully white. [`SharpLcd::flush()`] should be called afterwards.
    //todo? Maybe make this send the clear command as well so we don't manually write it all white
    pub fn clear(&mut self) {
        self.framebuffer = [[0; WIDTH/8]; HEIGHT];
    }

    /// Flush changes to the screen
    pub fn flush(&mut self) {
        todo!()
        // Might want to optimize with DMA, that'd require more params
    }

    /// Toggle the VCOM value. This should be done at least once per second to prevent burn-in.
    /// [`SharpLcd::flush()`] should be called afterwards.
    pub fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
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
        for pixel in pixels {
            // Check if on screen
            if self.bounding_box().contains(pixel.0) {
                // Convert to inverted u8
                let color_inv = pixel.1.is_off() as u8;
                // Write to framebuffer. Shouldn't panic because bounds have been checked
                self.framebuffer[pixel.0.y as usize][(pixel.0.x/8) as usize]
                    &= !(color_inv << (pixel.0.x % 8));
            }
        }

        Ok(())
    }
}