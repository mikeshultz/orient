/// Struct for Stepper configuration and control
use cortex_m_semihosting::hprintln;
use stm32f3xx_hal::gpio::gpiof::{PF10, PF6, PF9};
use stm32f3xx_hal::gpio::{gpiof, Alternate, Gpiof, OpenDrain, Output, Pin, U};
use stm32f3xx_hal::i2c;
use stm32f3xx_hal::pac;
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::pwm;
use stm32f3xx_hal::pwm::{PwmChannel, Tim15Ch2, WithPins};
use stm32f3xx_hal::rcc;

type Pf10Af3Pin = Pin<Gpiof, U<10_u8>, Alternate<OpenDrain, 3u8>>;

#[derive(Debug)]
pub enum CircularDirection {
    CW,
    CCW,
}

/*
not yet using enable pin because we don't have a 3rd 5v drain pin

pf6 - enable
pf9 - direction
pf10 - pulse
*/

pub struct Stepper {
    pub pin_enable: PF6<Output<OpenDrain>>,
    pub pin_direction: PF9<Output<OpenDrain>>,
    pub pwm_pulse: PwmChannel<Tim15Ch2, WithPins>,
}

impl Stepper {
    pub fn new<Pf6Mode, Pf9Mode, Pf10Mode>(
        pf6: PF6<Pf6Mode>,
        pf9: PF9<Pf9Mode>,
        pf10: PF10<Pf10Mode>,
        moder: &mut gpiof::MODER,
        otyper: &mut gpiof::OTYPER,
        afh: &mut gpiof::AFRH,
        // 2-channel timer w/ complimentary output
        tim15: pac::TIM15,
        clocks: rcc::Clocks,
    ) -> Result<Self, i2c::Error> {
        /*
         * Pinout:
         *   PF6 -> Enable
         *   PF9 -> Direction
         *   PF10 -> Pulse
         */
        hprintln!("Configuring PF6.").ok();
        let pin_enable = pf6.into_open_drain_output(moder, otyper);

        hprintln!("Configuring PF9.").ok();
        let pin_direction = pf9.into_open_drain_output(moder, otyper);

        hprintln!("Configuring PF10.").ok();
        let pin_pulse: Pf10Af3Pin = pf10.into_af_open_drain(moder, otyper, afh);

        // Setup hardware timer on PWM channel. Controller expects 13kHz
        hprintln!("Configuring PWM").ok();
        let (_, pwm_ch2) = pwm::tim15(tim15, 200, 13_000.Hz(), &clocks);
        let mut pwm_pulse = pwm_ch2.output_to_pf10(pin_pulse);
        let duty_cycle = pwm_pulse.get_max_duty() / 2; // 50%
        pwm_pulse.set_duty(duty_cycle);

        hprintln!("Done configuring Stepper.");

        Ok(Self {
            pin_enable,
            pin_direction,
            pwm_pulse,
        })
    }

    pub fn enable(&mut self) {
        self.pwm_pulse.enable();
    }

    pub fn disable(&mut self) {
        self.pwm_pulse.disable();
    }

    pub fn set_direction(&mut self, dir: CircularDirection) {
        match dir {
            CircularDirection::CW => self.pin_direction.set_high().ok(),
            _ => self.pin_direction.set_low().ok(),
        };
    }

    pub fn toggle_direction(&mut self) {
        if self.pin_direction.is_high().unwrap() {
            self.pin_direction.set_low().ok()
        } else {
            self.pin_direction.set_high().ok()
        };
    }
}
