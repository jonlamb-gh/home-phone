use crate::hal::gpio::*;
use crate::hal::prelude::*;
use crate::hal::time::{Duration, Instant};
use keypad::{keypad_new, keypad_struct, KeypadInput};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(25);
const LONGPRESS_DURATION: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum KeypadEvent {
    KeyPress(char),
    LongPress(char),
}

pub struct Keypad<INNER: KeypadDecomp> {
    states: KeyStateMatrix,
    inner: INNER,
}

pub trait KeypadDecomp {
    fn decompose<'a>(&'a self) -> [[KeypadInput<'a>; 3]; 4];
}

// TODO - make generic over any GPIO pins
keypad_struct! {
    pub struct KeypadInner {
        rows: (
          Pin5<Input<PullUp>>,
          Pin6<Input<PullUp>>,
          Pin13<Input<PullUp>>,
          Pin19<Input<PullUp>>,
        ),
        columns: (
            Pin17<Output<PushPull>>,
            Pin27<Output<PushPull>>,
            Pin22<Output<PushPull>>,
        ),
    }
}

impl KeypadInner {
    pub fn new(
        r0: Pin5<Input<PullUp>>,
        r1: Pin6<Input<PullUp>>,
        r2: Pin13<Input<PullUp>>,
        r3: Pin19<Input<PullUp>>,
        c0: Pin17<Output<PushPull>>,
        c1: Pin27<Output<PushPull>>,
        c2: Pin22<Output<PushPull>>,
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
pub struct KeyState {
    key: char,
    state: bool,
    pub prev_pressed: bool,
    last_db: Instant,
}

impl KeyState {
    pub fn new(key: char) -> Self {
        KeyState {
            key,
            state: false,
            prev_pressed: false,
            last_db: Instant { millis: 0 },
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
    use core::marker::PhantomData;
    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use log::trace;
    use void::Void;

    struct MockPin<MODE> {
        state: bool,
        _mode: PhantomData<MODE>,
    }

    impl<MODE> MockPin<MODE> {
        fn new(state: bool) -> Self {
            MockPin {
                state,
                _mode: PhantomData,
            }
        }
    }

    impl<MODE> InputPin for MockPin<Input<MODE>> {
        type Error = Void;

        fn is_high(&self) -> Result<bool, Self::Error> {
            self.is_low().map(|b| !b)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(self.state == false)
        }
    }

    impl<MODE> OutputPin for MockPin<Output<MODE>> {
        type Error = Void;

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            Ok(())
        }

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = true;
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
                MockPin<Output<PushPull>>,
                MockPin<Output<PushPull>>,
                MockPin<Output<PushPull>>,
            ),
        }
    }

    impl KeypadDecomp for MockKeypadInner {
        fn decompose<'a>(&'a self) -> [[KeypadInput<'a>; 3]; 4] {
            self.decompose()
        }
    }

    #[test_case]
    fn keypad_construction() {
        trace!("keypad_construction");

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
}
