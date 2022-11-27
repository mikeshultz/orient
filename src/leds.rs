/// Struct for LED configuration and control
use panic_semihosting as _;

use core::iter::FusedIterator;
use stm32f3xx_hal::gpio::gpioe::PEx;
use stm32f3xx_hal::gpio::{gpioe, Output, PushPull};
use switch_hal::{ActiveHigh, Switch};

pub type Led = Switch<PEx<Output<PushPull>>, ActiveHigh>;

pub enum LedId {
    Led3,  // North
    Led4,  // Northwest
    Led5,  // Northeast
    Led6,  // West
    Led7,  // East
    Led8,  // Southwest
    Led9,  // Southeast
    Led10, // South
}

// Implementation based on stm32f3-discovery crate but updated for new HAL
// version.  Ref: https://github.com/rubberduck203/stm32f3-discovery/blob/0ec450e604420c5e05591eba71b55e58fc9ee5fb/src/leds.rs
pub struct Leds {
    pub ld3: Led,
    pub ld4: Led,
    pub ld5: Led,
    pub ld6: Led,
    pub ld7: Led,
    pub ld8: Led,
    pub ld9: Led,
    pub ld10: Led,
}

impl Leds {
    pub fn new<PE8Mode, PE9Mode, PE10Mode, PE11Mode, PE12Mode, PE13Mode, PE14Mode, PE15Mode>(
        pe8: gpioe::PE8<PE8Mode>,
        pe9: gpioe::PE9<PE9Mode>,
        pe10: gpioe::PE10<PE10Mode>,
        pe11: gpioe::PE11<PE11Mode>,
        pe12: gpioe::PE12<PE12Mode>,
        pe13: gpioe::PE13<PE13Mode>,
        pe14: gpioe::PE14<PE14Mode>,
        pe15: gpioe::PE15<PE15Mode>,
        moder: &mut gpioe::MODER,
        otyper: &mut gpioe::OTYPER,
    ) -> Self {
        Leds {
            ld3: Switch::<_, ActiveHigh>::new(pe9.into_push_pull_output(moder, otyper).downgrade()),
            ld4: Switch::<_, ActiveHigh>::new(pe8.into_push_pull_output(moder, otyper).downgrade()),
            ld5: Switch::<_, ActiveHigh>::new(
                pe10.into_push_pull_output(moder, otyper).downgrade(),
            ),
            ld6: Switch::<_, ActiveHigh>::new(
                pe15.into_push_pull_output(moder, otyper).downgrade(),
            ),
            ld7: Switch::<_, ActiveHigh>::new(
                pe11.into_push_pull_output(moder, otyper).downgrade(),
            ),
            ld8: Switch::<_, ActiveHigh>::new(
                pe14.into_push_pull_output(moder, otyper).downgrade(),
            ),
            ld9: Switch::<_, ActiveHigh>::new(
                pe12.into_push_pull_output(moder, otyper).downgrade(),
            ),
            ld10: Switch::<_, ActiveHigh>::new(
                pe13.into_push_pull_output(moder, otyper).downgrade(),
            ),
        }
    }

    /// Return a mutable iterator of all configured LEDs
    pub fn iter_mut(&mut self) -> LedsMutIterator {
        LedsMutIterator::new(self)
    }
}

impl<'a> IntoIterator for &'a mut Leds {
    type Item = &'a mut Led;
    type IntoIter = LedsMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

const ITERATOR_SIZE: usize = 8;

pub struct LedsMutIterator<'a> {
    index: usize,
    index_back: usize,
    leds: &'a mut Leds,
}

impl<'a> LedsMutIterator<'a> {
    fn new(leds: &'a mut Leds) -> Self {
        LedsMutIterator {
            index: 0,
            index_back: ITERATOR_SIZE,
            leds,
        }
    }

    fn len(&self) -> usize {
        self.index_back - self.index
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> Iterator for LedsMutIterator<'a> {
    type Item = &'a mut Led;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            None
        } else {
            let current = unsafe {
                //Safety: Each branch is only executed once,
                // and only if there are elements left to be returned,
                // so we can not possibly alias a mutable reference.
                // This depends on DoubleEndedIterator and ExactSizedIterator being implemented correctly.
                // If len() does not return the correct number of remaining elements,
                // this becomes unsound.
                match self.index {
                    0 => Some(&mut *(&mut self.leds.ld3 as *mut _)),  //N
                    1 => Some(&mut *(&mut self.leds.ld5 as *mut _)),  //NE
                    2 => Some(&mut *(&mut self.leds.ld7 as *mut _)),  //E
                    3 => Some(&mut *(&mut self.leds.ld9 as *mut _)),  //SE
                    4 => Some(&mut *(&mut self.leds.ld10 as *mut _)), //S
                    5 => Some(&mut *(&mut self.leds.ld8 as *mut _)),  //SW
                    6 => Some(&mut *(&mut self.leds.ld6 as *mut _)),  //W
                    7 => Some(&mut *(&mut self.leds.ld4 as *mut _)),  //NW
                    _ => None,
                }
            };
            self.index += 1;
            current
        }
    }

    // Because we implement ExactSizedIterator, we need to ensure size_hint returns the right length
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.size_hint()
    }
}

impl<'a> DoubleEndedIterator for LedsMutIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            None
        } else {
            let current = unsafe {
                //Safety: Each branch is only executed once,
                // and only if there are elements left to be returned,
                // so we can not possibly alias a mutable reference.
                // This depends on Iterator and ExactSizedIterator being implemented correctly.
                // If len() does not return the correct number of remaining elements,
                // this becomes unsound.
                match self.index_back {
                    // Because we're going backwards and index_back is a usize,
                    // We use a one based index so we don't go negative
                    0 => None,                                        //done
                    1 => Some(&mut *(&mut self.leds.ld3 as *mut _)),  //N
                    2 => Some(&mut *(&mut self.leds.ld5 as *mut _)),  //NE
                    3 => Some(&mut *(&mut self.leds.ld7 as *mut _)),  //E
                    4 => Some(&mut *(&mut self.leds.ld9 as *mut _)),  //SE
                    5 => Some(&mut *(&mut self.leds.ld10 as *mut _)), //S
                    6 => Some(&mut *(&mut self.leds.ld8 as *mut _)),  //SW
                    7 => Some(&mut *(&mut self.leds.ld6 as *mut _)),  //W
                    8 => Some(&mut *(&mut self.leds.ld4 as *mut _)),  //NW
                    _ => None,                                        //can't happen
                }
            };
            self.index_back -= 1;
            current
        }
    }
}

impl<'a> ExactSizeIterator for LedsMutIterator<'a> {
    fn len(&self) -> usize {
        self.len()
    }
}

///Marker trait that indicates LedsMutIterator never starts returning Some after returning None
impl<'a> FusedIterator for LedsMutIterator<'a> {}
