//! State machine stuff

use core::fmt::Formatter;

use stm32l0xx_hal as hal;

pub mod clock;

/// State shared by the different UI modes
#[derive(Debug)]
pub struct SharedState {

}

impl SharedState {
    pub fn new() -> Self {
        Self {

        }
    }
}

/// Resources shared by the different UI modes
pub struct Resources {
    pub rtc: hal::rtc::Rtc,
    //todo: make generic over gfx::DrawTarget
    pub display: crate::peripherals::display::SharpLcd<
        hal::spi::Spi<
            hal::pac::SPI1,
            (
                hal::gpio::gpioa::PA5<hal::gpio::Analog>,
                hal::gpio::gpioa::PA6<hal::gpio::Analog>,
                hal::gpio::gpioa::PA7<hal::gpio::Analog>
            )
        >,
        hal::gpio::gpioa::PA4<hal::gpio::Output<hal::gpio::PushPull>>
    >
}
impl core::fmt::Debug for Resources {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Resources")
            .field("display", &self.display)
            .finish_non_exhaustive() // since some peripherals don't have Debug
    }
}

/// The individual UI modes, such as clock, alarms, etc.
#[derive(Debug)]
pub enum UiMode<'a> {
    Clock(clock::ClockMode<'a>)
}
impl<'a> Default for UiMode<'a> {
    fn default() -> Self {
        Self::Clock(clock::ClockMode::new())
    }
}

impl<'a> UiMode<'a> {
    /// Wrapper function to dispatch to the current mode's `update()` function
    pub fn update(&mut self, resources: &mut Resources, shared_state: &mut SharedState) -> Option<Self> {
        match self {
            Self::Clock(x) => x.update(resources, shared_state)
        }
    }

    /// Wrapper function to dispatch to the current mode's `draw()` function
    pub fn draw(&self, resources: &mut Resources, shared_state: &SharedState) {
        match self {
            Self::Clock(x) => x.draw(resources, shared_state)
        }
    }
}

#[derive(Debug)]
pub struct State<'a> {
    /// Shared resources, such as hardware peripherals
    resources: Resources,

    /// Shared state, such as configuration data
    shared_state: SharedState,

    /// The current UI state
    mode: UiMode<'a>
}

impl<'a> State<'a> {
    pub fn new(resources: Resources, shared_state: SharedState) -> Self {
        Self {
            resources,
            shared_state,
            mode: UiMode::default()
        }
    }

    /// Update the current state. Should be called periodically.
    pub fn update(&mut self) {
        // If the update switches state, switch to that state otherwise do nothing
        match self.mode.update(&mut self.resources, &mut self.shared_state) {
            Some(mode) => self.mode = mode,
            None => ()
        }
    }

    /// Redraw the display
    pub fn draw(&mut self) {
        self.mode.draw(&mut self.resources, &self.shared_state)
    }
}