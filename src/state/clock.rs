//! The clock state.

use embedded_graphics::{
    self as gfx,
    prelude::*,
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor
};
use chrono::prelude::*;
use core::fmt::Write;

use crate::state::{UiMode, SharedState, Resources};

#[derive(Debug)]
pub struct ClockMode<'a> {
    text_style: MonoTextStyle<'a, BinaryColor>
}

impl<'a> ClockMode<'a> {
    pub fn new() -> Self {
        Self {
            text_style: gfx::mono_font::MonoTextStyle::new(
                &gfx::mono_font::iso_8859_1::FONT_4X6,
                gfx::pixelcolor::BinaryColor::On
            )
        }
    }

    pub fn update(&mut self, _resources: &mut Resources, _shared_state: &mut SharedState) -> Option<UiMode<'a>> {
        // Stay in this state forever
        None
    }

    pub fn draw(&self, resources: &mut Resources, _shared_state: &SharedState) {
        resources.display.clear();
        // Draw time

        let time = resources.rtc.now().time();
        // Create buffer for time string (should fit in 8 chars)
        let mut time_str = arrayvec::ArrayString::<8>::new();
        // Format time string
        write!(time_str, "{:02}:{:02}:{:02}", time.hour(), time.minute(), time.second()).unwrap();
        // Draw text to framebuffer
        // Cannot error
        let _ = gfx::text::Text::new(&time_str, Point::new(1, 1), self.text_style).draw(&mut resources.display);

        // Toggle VCOM and flush
        resources.display.toggle_vcom();
        resources.display.flush().unwrap();
    }
}