use stm32l0xx_hal::{
    prelude::*,
    pwm::{
        Instance,
        C1,
        Pin,
        Pwm,
        Assigned,
        Unassigned
    },
    rcc::Rcc
};

pub struct Buzzer<T, P> {
    pwm: Pwm<T, C1, Assigned<P>>
}

impl<T: Instance, P: Pin<T, C1>> Buzzer<T, P> {
    pub fn new(pwm: Pwm<T, C1, Unassigned>, pin: P) -> Self {
        let mut pwm = pwm.assign(pin);
        pwm.set_duty(0);

        Self {
            pwm
        }
    }

    /// Sounds a normal-pitched (2 kHz) beep, for regular alerts
    pub fn beep(&mut self, rcc: &Rcc) {
        self.pwm.set_frequency(2000.Hz(), rcc);
        todo!()
    }

    /// Sounds a high-pitched (4 kHz) beep, for certain events (like looping around in a menu)
    pub fn beep_high(&mut self, rcc: &Rcc) {
        self.pwm.set_frequency(4000.Hz(), rcc);
        todo!()
    }
}
