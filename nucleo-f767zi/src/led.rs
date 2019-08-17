use core::ops;
use crate::hal::gpio::gpiob::{PBx, PB0, PB14, PB7};
use crate::hal::gpio::{Output, PushPull};
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};

// Green
pub type LED1 = PB0<Output<PushPull>>;

// Blue
pub type LED2 = PB7<Output<PushPull>>;

// Red
pub type LED3 = PB14<Output<PushPull>>;

pub enum Color {
    // Green / LED1
    Green,
    // Blue / LED2
    Blue,
    // Red / LED3
    Red,
}

/// Array of all the user LEDs on the board
pub struct Leds {
    leds: [Led; 3],
}

impl Leds {
    /// Initializes all the user LEDs
    pub fn new(r: LED3, g: LED1, b: LED2) -> Self {
        Leds {
            leds: [g.into(), b.into(), r.into()],
        }
    }
}

impl ops::Deref for Leds {
    type Target = [Led];

    fn deref(&self) -> &[Led] {
        &self.leds
    }
}

impl ops::DerefMut for Leds {
    fn deref_mut(&mut self) -> &mut [Led] {
        &mut self.leds
    }
}

impl ops::Index<usize> for Leds {
    type Output = Led;

    fn index(&self, i: usize) -> &Led {
        &self.leds[i]
    }
}

impl ops::Index<Color> for Leds {
    type Output = Led;

    fn index(&self, c: Color) -> &Led {
        &self.leds[c as usize]
    }
}

impl ops::IndexMut<usize> for Leds {
    fn index_mut(&mut self, i: usize) -> &mut Led {
        &mut self.leds[i]
    }
}

impl ops::IndexMut<Color> for Leds {
    fn index_mut(&mut self, c: Color) -> &mut Led {
        &mut self.leds[c as usize]
    }
}

pub struct Led {
    pbx: PBx<Output<PushPull>>,
}

macro_rules! ctor {
    ($($ldx:ident),+) => {
        $(
            impl Into<Led> for $ldx {
                fn into(self) -> Led {
                    Led {
                        pbx: self.downgrade(),
                    }
                }
            }
        )+
    }
}

ctor!(LED1, LED2, LED3);

impl Led {
    /// Turns the LED off
    pub fn off(&mut self) {
        self.pbx.set_low().unwrap();
    }

    /// Turns the LED on
    pub fn on(&mut self) {
        self.pbx.set_high().unwrap();
    }

    /// Toggles the LED
    pub fn toggle(&mut self) {
        self.pbx.toggle().unwrap();
    }
}
