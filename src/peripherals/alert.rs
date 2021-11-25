use stm32l0xx_hal::{
    self as hal,
    prelude::*,
    pwm::{
        Timer,
        Instance
    },
    rcc::Rcc
};

pub struct Buzzer<T> {
    timer: Timer<T>
}

impl<T: Instance> Buzzer<T> {
    pub fn new(timer: Timer<T>) -> Self {
        Self {
            timer
        }
    }

    /// Sounds a normal-pitched (2 kHz) beep, for regular alerts
    pub fn beep(&mut self, rcc: &Rcc) {
        self.timer.set_frequency(2000.Hz(), rcc);
        todo!()
    }

    /// Sounds a high-pitched (4 kHz) beep, for certain events (like looping around in a menu)
    pub fn beep_high(&mut self, rcc: &Rcc) {
        self.timer.set_frequency(4000.Hz(), rcc);
        todo!()
    }
}
