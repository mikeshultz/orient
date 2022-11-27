#![no_std]
/// Primary library interface for orient used for STM32F3DISCOVERY device
/// control and configuration.
use panic_semihosting as _;

use cortex_m::asm;
use cortex_m_semihosting::{heprintln, hprintln};
use stm32f3xx_hal::pac;
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::rcc;
use stm32f3xx_hal::time::rate::*;
use switch_hal::OutputSwitch;

pub mod compass;
pub mod geo;
pub mod leds;
pub mod stepper;

#[derive(Debug)]
pub enum Error {
    Hardware,
}

pub struct RangeF32 {
    start: f32,
    end: f32, // exclusive
}

impl RangeF32 {
    pub fn contains(self: &Self, n: f32) -> bool {
        n >= self.start && n < self.end
    }
}

/// The struct representing the entire device. All operations and memory writes
/// should generally be done through this struct.
pub struct ConfiguredDevice {
    pub clocks: rcc::Clocks,
    pub leds: leds::Leds,
    pub compass: Option<compass::Compass>,
    pub stepper: Option<stepper::Stepper>,
}

impl ConfiguredDevice {
    pub fn new(device: pac::Peripherals) -> Self {
        let mut rcc = device.RCC.constrain();
        let mut flash = device.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let clk = clocks.sysclk().0;

        let mut gpiob = device.GPIOB.split(&mut rcc.ahb);
        let mut gpioe = device.GPIOE.split(&mut rcc.ahb);
        let mut gpiof = device.GPIOF.split(&mut rcc.ahb);

        hprintln!("Configring LEDs...").ok();

        hprintln!("Configring LEDs...").ok();
        let mut _leds = leds::Leds::new(
            gpioe.pe8,
            gpioe.pe9,
            gpioe.pe10,
            gpioe.pe11,
            gpioe.pe12,
            gpioe.pe13,
            gpioe.pe14,
            gpioe.pe15,
            &mut gpioe.moder,
            &mut gpioe.otyper,
        );

        hprintln!("Configring Compass...").ok();
        let compass = compass::Compass::new(
            gpiob.pb6,
            gpiob.pb7,
            &mut gpiob.moder,
            &mut gpiob.otyper,
            &mut gpiob.afrl,
            device.I2C1,
            clocks,
            &mut rcc.apb1,
        )
        .ok();

        hprintln!("Configring Stepper...").ok();
        let stepper = stepper::Stepper::new(
            gpiof.pf6,
            gpiof.pf9,
            gpiof.pf10,
            &mut gpiof.moder,
            &mut gpiof.otyper,
            &mut gpiof.afrh,
            device.TIM15,
            clocks,
        )
        .ok();

        Self {
            clocks,
            leds: _leds,
            compass,
            stepper,
        }
    }

    /// Sleep the thread for N milliseconds
    pub fn delay(self: &Self, ms: u32) {
        // TODO: refactor to use systick_monotoic/fugit Duration?
        let freq = self.clocks.sysclk().integer();
        let cycles_per_ms: u32 = freq / 1000;
        asm::delay(cycles_per_ms * ms);
    }

    /// Turn off all configured LEDs
    pub fn leds_clear(self: &mut Self) {
        for led in self.leds.iter_mut() {
            led.off().ok();
        }
    }

    /// Turn the given LED on
    pub fn led_on(self: &mut Self, led_id: leds::LedId) {
        (match led_id {
            leds::LedId::Led3 => &mut self.leds.ld3,
            leds::LedId::Led4 => &mut self.leds.ld4,
            leds::LedId::Led5 => &mut self.leds.ld5,
            leds::LedId::Led6 => &mut self.leds.ld6,
            leds::LedId::Led7 => &mut self.leds.ld7,
            leds::LedId::Led8 => &mut self.leds.ld8,
            leds::LedId::Led9 => &mut self.leds.ld9,
            leds::LedId::Led10 => &mut self.leds.ld10,
        })
        .on()
        .ok();
    }

    /// Turn the given LED off
    pub fn led_off(self: &mut Self, led_id: leds::LedId) {
        (match led_id {
            leds::LedId::Led3 => &mut self.leds.ld3,
            leds::LedId::Led4 => &mut self.leds.ld4,
            leds::LedId::Led5 => &mut self.leds.ld5,
            leds::LedId::Led6 => &mut self.leds.ld6,
            leds::LedId::Led7 => &mut self.leds.ld7,
            leds::LedId::Led8 => &mut self.leds.ld8,
            leds::LedId::Led9 => &mut self.leds.ld9,
            leds::LedId::Led10 => &mut self.leds.ld10,
        })
        .off()
        .ok();
    }

    /// Flash the top LED repeatedly to indicate an error
    pub fn flash_error(self: &mut Self) -> ! {
        loop {
            // TODO: Refactor for RTIC delay via task spawning
            self.led_on(leds::LedId::Led3);
            self.delay(100);
            self.led_off(leds::LedId::Led3);
            self.delay(100);
        }
    }

    /// Read compass bearing toward north
    pub fn bearing_north(self: &mut Self) -> f32 {
        match &mut self.compass {
            Some(compass) => compass.bearing_north(),
            None => {
                heprintln!("compass: compass not configured").ok();
                self.flash_error();
            }
        }
    }

    /// Turn on the stepper PWM signal
    pub fn stepper_enable(self: &mut Self) {
        match &mut self.stepper {
            Some(stepper) => {
                stepper.enable();
            }
            None => {
                heprintln!("stepper_enable: stepper not configured").ok();
                self.flash_error();
            }
        }
    }

    /// Turn off the stepper PWM signal
    pub fn stepper_disable(self: &mut Self) {
        match &mut self.stepper {
            Some(stepper) => {
                stepper.disable();
            }
            None => {
                heprintln!("stepper_disable: stepper not configured").ok();
                self.flash_error();
            }
        }
    }

    /// Set the driection of the stepper
    pub fn stepper_set_direction(self: &mut Self, dir: stepper::CircularDirection) {
        match &mut self.stepper {
            Some(stepper) => {
                stepper.set_direction(dir);
            }
            None => {
                heprintln!("stepper_set_direction: stepper not configured").ok();
                self.flash_error();
            }
        }
    }

    /// Toggle the driection of the stepper
    pub fn stepper_toggle_direction(self: &mut Self) {
        match &mut self.stepper {
            Some(stepper) => {
                stepper.toggle_direction();
            }
            None => {
                heprintln!("stepper_set_direction: stepper not configured").ok();
                self.flash_error();
            }
        }
    }
}

pub fn display_bearing(board: &mut ConfiguredDevice, bearing: f32) -> Result<(), Error> {
    board.leds_clear();

    // LED CW order start from N 3, 5, 7, 9, 10, 8, 6, 4
    match bearing {
        x if (RangeF32 {
            start: -22.0,
            end: 22.0,
        })
        .contains(x) =>
        {
            board.leds.ld3.on().ok()
        }
        // NE
        x if (RangeF32 {
            start: 22.0,
            end: 67.0,
        })
        .contains(x) =>
        {
            board.leds.ld5.on().ok()
        }
        // E
        x if (RangeF32 {
            start: 67.0,
            end: 112.0,
        })
        .contains(x) =>
        {
            board.leds.ld7.on().ok()
        }
        // SE
        x if (RangeF32 {
            start: 112.0,
            end: 157.0,
        })
        .contains(x) =>
        {
            board.leds.ld9.on().ok()
        }
        // South
        x if (RangeF32 {
            start: 157.0,
            end: 180.0,
        })
        .contains(x) =>
        {
            board.leds.ld10.on().ok()
        }
        x if (RangeF32 {
            start: -180.0,
            end: -157.0,
        })
        .contains(x) =>
        {
            board.leds.ld10.on().ok()
        }
        // SW
        x if (RangeF32 {
            start: -157.0,
            end: -112.0,
        })
        .contains(x) =>
        {
            board.leds.ld8.on().ok()
        }
        // W
        x if (RangeF32 {
            start: -112.0,
            end: -67.0,
        })
        .contains(x) =>
        {
            board.leds.ld6.on().ok()
        }
        // NW
        x if (RangeF32 {
            start: -67.0,
            end: -22.0,
        })
        .contains(x) =>
        {
            board.leds.ld4.on().ok()
        }
        x => {
            heprintln!("Reading {} is out of range", x).unwrap();
            board.flash_error();
        }
    };

    Ok(())
}
