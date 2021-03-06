mod event_buffer;

pub use crate::keypad::event_buffer::{EventBuffer, EventBufferMode};

use crate::hal::gpio::{gpiob, Input, OpenDrain, Output, PullUp};
use crate::hal::prelude::*;
use crate::time::{Duration, Instant};
use keypad::{keypad_new, keypad_struct, KeypadInput};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(25);
const LONGPRESS_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum KeypadEvent {
    KeyPress(char),
    LongPress(char),
}

impl KeypadEvent {
    pub fn as_char(&self) -> char {
        match *self {
            KeypadEvent::KeyPress(c) | KeypadEvent::LongPress(c) => c,
        }
    }
}

pub struct Keypad<INNER: KeypadDecomp> {
    states: KeyStateMatrix,
    inner: INNER,
}

pub trait KeypadDecomp {
    fn decompose<'a>(&'a self) -> [[KeypadInput<'a>; 3]; 4];
}

// TODO - make generic over any GPIO pins
// - need to pick real pins, these are just made up for now
keypad_struct! {
    pub struct KeypadInner {
        rows: (
          gpiob::PB0<Input<PullUp>>,
          gpiob::PB1<Input<PullUp>>,
          gpiob::PB2<Input<PullUp>>,
          gpiob::PB3<Input<PullUp>>,
        ),
        columns: (
            gpiob::PB4<Output<OpenDrain>>,
            gpiob::PB5<Output<OpenDrain>>,
            gpiob::PB6<Output<OpenDrain>>,
        ),
    }
}

impl KeypadInner {
    pub fn new(
        r0: gpiob::PB0<Input<PullUp>>,
        r1: gpiob::PB1<Input<PullUp>>,
        r2: gpiob::PB2<Input<PullUp>>,
        r3: gpiob::PB3<Input<PullUp>>,
        c0: gpiob::PB4<Output<OpenDrain>>,
        c1: gpiob::PB5<Output<OpenDrain>>,
        c2: gpiob::PB6<Output<OpenDrain>>,
    ) -> Self {
        keypad_new!(KeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        })
    }
}

impl KeypadDecomp for KeypadInner {
    fn decompose<'a>(&'a self) -> [[KeypadInput<'a>; 3]; 4] {
        self.decompose()
    }
}

impl<INNER> Keypad<INNER>
where
    INNER: KeypadDecomp,
{
    pub fn new(inner: INNER) -> Self {
        Keypad {
            states: [
                [KeyState::new('1'), KeyState::new('2'), KeyState::new('3')],
                [KeyState::new('4'), KeyState::new('5'), KeyState::new('6')],
                [KeyState::new('7'), KeyState::new('8'), KeyState::new('9')],
                [KeyState::new('*'), KeyState::new('0'), KeyState::new('#')],
            ],
            inner,
        }
    }

    pub fn read(&mut self, time: &Instant) -> Option<KeypadEvent> {
        let keys = self.inner.decompose();
        for (row_index, row) in keys.iter().enumerate() {
            for (col_index, key) in row.iter().enumerate() {
                let pressed = key.is_low().unwrap();
                let changed = self.states[row_index][col_index].set(time, pressed);
                let (_db_pressed, prev_pressed) = self.states[row_index][col_index].pressed(time);
                let long_pressed = self.states[row_index][col_index].long_pressed(time);
                if changed && prev_pressed {
                    let c = self.states[row_index][col_index].key();

                    // Clear all of the states, not tracking multi-key presses
                    for s in self.states.iter_mut().flat_map(|r| r.iter_mut()) {
                        s.last_db = *time;
                        s.prev_pressed = false;
                    }

                    return Some(match long_pressed {
                        false => KeypadEvent::KeyPress(c),
                        true => KeypadEvent::LongPress(c),
                    });
                }
            }
        }

        None
    }
}

type KeyStateMatrix = [[KeyState; 3]; 4];

#[derive(Debug)]
struct KeyState {
    key: char,
    state: bool,
    prev_pressed: bool,
    last_db: Instant,
}

impl KeyState {
    pub fn new(key: char) -> Self {
        KeyState {
            key,
            state: false,
            prev_pressed: false,
            last_db: Instant::new(0, 0),
        }
    }

    pub fn key(&self) -> char {
        self.key
    }

    // (pressed, prev_pressed)
    pub fn pressed(&mut self, time: &Instant) -> (bool, bool) {
        let prev = self.prev_pressed;
        self.prev_pressed = if (*time - self.last_db) < DEBOUNCE_DURATION {
            false
        } else {
            self.state
        };
        (self.prev_pressed, prev)
    }

    pub fn long_pressed(&mut self, time: &Instant) -> bool {
        if (*time - self.last_db) < LONGPRESS_DURATION {
            false
        } else {
            true
        }
    }

    /// Returns true if state changed
    pub fn set(&mut self, time: &Instant, state: bool) -> bool {
        let prev = self.state;
        self.state = state;
        let changed = prev != self.state;
        if changed && state {
            self.last_db = *time;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;
    use core::marker::PhantomData;
    use embedded_hal::digital::v2::{InputPin, OutputPin};

    struct MockPin<MODE> {
        is_low: bool,
        _mode: PhantomData<MODE>,
    }

    impl<MODE> MockPin<MODE> {
        fn new(is_low: bool) -> Self {
            MockPin {
                is_low,
                _mode: PhantomData,
            }
        }
    }

    impl<MODE> InputPin for MockPin<Input<MODE>> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            self.is_low().map(|b| !b)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.is_low)
        }
    }

    impl<MODE> OutputPin for MockPin<Output<MODE>> {
        type Error = Infallible;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.is_low = false;
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.is_low = true;
            Ok(())
        }
    }

    keypad_struct! {
        struct MockKeypadInner {
            rows: (
              MockPin<Input<PullUp>>,
              MockPin<Input<PullUp>>,
              MockPin<Input<PullUp>>,
              MockPin<Input<PullUp>>,
            ),
            columns: (
                MockPin<Output<OpenDrain>>,
                MockPin<Output<OpenDrain>>,
                MockPin<Output<OpenDrain>>,
            ),
        }
    }

    impl KeypadDecomp for MockKeypadInner {
        fn decompose<'a>(&'a self) -> [[KeypadInput<'a>; 3]; 4] {
            self.decompose()
        }
    }

    #[test]
    fn construction() {
        let r0 = MockPin::new(false);
        let r1 = MockPin::new(false);
        let r2 = MockPin::new(false);
        let r3 = MockPin::new(false);
        let c0 = MockPin::new(false);
        let c1 = MockPin::new(false);
        let c2 = MockPin::new(false);

        let inner = keypad_new!(MockKeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        });

        let mut keypad = Keypad::new(inner);

        let t = Instant::from_millis(0);
        assert_eq!(keypad.read(&t), None);
    }

    #[test]
    fn short_press() {
        let r0 = MockPin::new(true);
        let r1 = MockPin::new(false);
        let r2 = MockPin::new(false);
        let r3 = MockPin::new(false);
        let c0 = MockPin::new(false);
        let c1 = MockPin::new(false);
        let c2 = MockPin::new(false);

        let inner_set = keypad_new!(MockKeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        });

        let r0 = MockPin::new(false);
        let r1 = MockPin::new(false);
        let r2 = MockPin::new(false);
        let r3 = MockPin::new(false);
        let c0 = MockPin::new(false);
        let c1 = MockPin::new(false);
        let c2 = MockPin::new(false);

        let inner_clr = keypad_new!(MockKeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        });

        let mut keypad = Keypad::new(inner_set);

        let t_0 = Instant::from_millis(0);
        assert_eq!(keypad.read(&t_0), None);

        let t = t_0 + (DEBOUNCE_DURATION / 2);
        assert_eq!(keypad.read(&t), None);

        let t = t_0 + (DEBOUNCE_DURATION - Duration::from_millis(1));
        assert_eq!(keypad.read(&t), None);

        let t = t_0 + DEBOUNCE_DURATION;
        assert_eq!(keypad.read(&t), None);

        keypad.inner = inner_clr;
        let t = t_0 + (DEBOUNCE_DURATION + Duration::from_millis(1));
        assert_eq!(keypad.read(&t), Some(KeypadEvent::KeyPress('1')));

        let t = t_0 + (DEBOUNCE_DURATION + Duration::from_millis(2));
        assert_eq!(keypad.read(&t), None);
    }

    #[test]
    fn long_press() {
        let r0 = MockPin::new(true);
        let r1 = MockPin::new(false);
        let r2 = MockPin::new(false);
        let r3 = MockPin::new(false);
        let c0 = MockPin::new(false);
        let c1 = MockPin::new(false);
        let c2 = MockPin::new(false);

        let inner_set = keypad_new!(MockKeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        });

        let r0 = MockPin::new(false);
        let r1 = MockPin::new(false);
        let r2 = MockPin::new(false);
        let r3 = MockPin::new(false);
        let c0 = MockPin::new(false);
        let c1 = MockPin::new(false);
        let c2 = MockPin::new(false);

        let inner_clr = keypad_new!(MockKeypadInner {
            rows: (r0, r1, r2, r3,),
            columns: (c0, c1, c2,),
        });

        let mut keypad = Keypad::new(inner_set);

        let t_0 = Instant::from_millis(0);
        assert_eq!(keypad.read(&t_0), None);

        let t = t_0 + (DEBOUNCE_DURATION / 2);
        assert_eq!(keypad.read(&t), None);

        let t = t_0 + (DEBOUNCE_DURATION + Duration::from_millis(1));
        assert_eq!(keypad.read(&t), None);

        let t = t_0 + (LONGPRESS_DURATION / 2);
        assert_eq!(keypad.read(&t), None);

        keypad.inner = inner_clr;
        let t = t_0 + LONGPRESS_DURATION;
        assert_eq!(keypad.read(&t), Some(KeypadEvent::LongPress('1')));

        let t = t_0 + (LONGPRESS_DURATION + Duration::from_millis(1));
        assert_eq!(keypad.read(&t), None);
    }

    #[test]
    fn events() {
        let short = KeypadEvent::KeyPress('C');
        let long = KeypadEvent::LongPress('C');
        assert_eq!(short.as_char(), long.as_char());
    }
}
