/// Struct for Compass configuration and control
use panic_semihosting as _;

use accelerometer::vector::I32x3;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagContinuous;
use lsm303agr::{Lsm303agr, MagOutputDataRate};
use stm32f3xx_hal::gpio::{gpiob, OpenDrain, AF4};
use stm32f3xx_hal::i2c;
use stm32f3xx_hal::pac;
use stm32f3xx_hal::prelude::*;
use stm32f3xx_hal::rcc;

use crate::geo::Point2D;

pub type Lsm303 = Lsm303agr<
    I2cInterface<i2c::I2c<pac::I2C1, (gpiob::PB6<AF4<OpenDrain>>, gpiob::PB7<AF4<OpenDrain>>)>>,
    MagContinuous,
>;

/// STMF3DISCOVERY Rev E is currently unsupported by stm32f3-discovery crate
/// so I'm implementing an lsm303agr driver based on the PR here:
/// https://github.com/rubberduck203/stm32f3-discovery/pull/43
pub struct Compass {
    lsm303: Lsm303,
}

impl Compass {
    pub fn new<Pb6Mode, Pb7Mode>(
        pb6: gpiob::PB6<Pb6Mode>,
        pb7: gpiob::PB7<Pb7Mode>,
        moder: &mut gpiob::MODER,
        otyper: &mut gpiob::OTYPER,
        afl: &mut gpiob::AFRL,
        i2c1: pac::I2C1,
        clocks: rcc::Clocks,
        advanced_periph_bus: &mut rcc::APB1,
    ) -> Result<Self, i2c::Error> {
        /*
         * Pinout:
         * PB6 -> SCL (clock)
         * PB7 -> SDA (data)
         * PE2 -> DRDY (magnometer data ready)
         * PE4 -> INT1 (configurable interrupt 1)
         * PE5 -> INT2 (configurable interrupt 2)
         */
        // TODO: Fiure out how to deal with the depreciation.  The i2c API
        // requires these at the moment?
        //let scl = pb6.into_open_drain_output(moder, otyper);
        //let sda = pb7.into_open_drain_output(moder, otyper);
        let scl = pb6.into_af4_open_drain(moder, otyper, afl);
        let sda = pb7.into_af4_open_drain(moder, otyper, afl);
        let i2ci = i2c::I2c::new(i2c1, (scl, sda), 400_000.Hz(), clocks, advanced_periph_bus);
        let mut lsm303agr = Lsm303agr::new_with_i2c(i2ci);
        lsm303agr.init().unwrap();
        lsm303agr.set_mag_odr(MagOutputDataRate::Hz20).unwrap();

        match lsm303agr.into_mag_continuous() {
            Ok(lsm303) => Ok(Self { lsm303 }),
            Err(_e) => Err(i2c::Error::Nack),
        }
    }

    /// Reading returned in nT (nanotesla)
    pub fn mag_raw(&mut self) -> Result<I32x3, i2c::Error> {
        let reading = self.lsm303.mag_data().unwrap();
        Ok(I32x3::new(reading.x, reading.y, reading.z))
    }

    /// Reading returned in nT (nanotesla)
    pub fn bearing_north(&mut self) -> f32 {
        let reading = self.lsm303.mag_data().unwrap();
        let point = Point2D::new(reading.x as f32, reading.y as f32);
        // Flip
        point.bearing_north() * -1.0
    }

    /// Consume the Compass and return the underlying lsm303agr
    pub fn into_lsm303agr(self) -> Lsm303 {
        self.lsm303
    }
}
