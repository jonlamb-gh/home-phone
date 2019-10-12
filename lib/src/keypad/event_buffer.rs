use crate::keypad::KeypadEvent;
use heapless::consts::U128;
use heapless::String;
use log::debug;

// TODO - redo these variants
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EventBufferMode {
    /// Buffer short-press digits until long-press '#' to produce a PhoneNumber
    WaitForUserDial,
    /// Each short-press digit is treated as a DTMF event, also buffered
    /// for history/logging
    Dtmf,
}

impl Default for EventBufferMode {
    fn default() -> Self {
        EventBufferMode::WaitForUserDial
    }
}

#[derive(Debug, Clone)]
pub struct EventBuffer {
    mode: EventBufferMode,
    buffer: Storage,
}

type Storage = String<U128>;

impl EventBuffer {
    pub fn new() -> Self {
        EventBuffer {
            mode: EventBufferMode::default(),
            buffer: Storage::new(),
        }
    }

    pub fn mode(&self) -> EventBufferMode {
        self.mode
    }

    pub fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn push(&mut self, mode: EventBufferMode, event: KeypadEvent) -> bool {
        if mode != self.mode {
            self.clear();
        }

        self.mode = mode;

        match self.mode {
            EventBufferMode::WaitForUserDial => match event {
                KeypadEvent::KeyPress(c) => {
                    self.buffer
                        .push(c)
                        .unwrap_or_else(|_| debug!("EventBuffer full"));
                    false
                }
                KeypadEvent::LongPress(c) => c == '#',
            },
            EventBufferMode::Dtmf => {
                if let KeypadEvent::KeyPress(c) = event {
                    self.buffer
                        .push(c)
                        .unwrap_or_else(|_| debug!("EventBuffer full"));
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl AsRef<str> for EventBuffer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::trace;

    #[test_case]
    fn clearing() {
        trace!("clearing");
        let mut eb = EventBuffer::new();

        let number = "22233334444";
        for c in number.chars() {
            assert_eq!(
                false,
                eb.push(EventBufferMode::WaitForUserDial, KeypadEvent::KeyPress(c))
            );
        }
        assert_eq!(eb.buffer.len(), 11);
        eb.clear();
        assert_eq!(eb.buffer.len(), 0);

        for c in number.chars() {
            assert_eq!(
                true,
                eb.push(EventBufferMode::Dtmf, KeypadEvent::KeyPress(c))
            );
        }
        assert_eq!(eb.buffer.len(), 11);
        eb.clear();
        assert_eq!(eb.buffer.len(), 0);
    }

    #[test_case]
    fn mode_changes_clear() {
        trace!("mode_changes_clear");
        let mut eb = EventBuffer::new();
        assert_eq!(eb.buffer.len(), 0);

        for c in "12345".chars() {
            assert_eq!(
                false,
                eb.push(EventBufferMode::WaitForUserDial, KeypadEvent::KeyPress(c))
            );
        }
        assert_eq!(eb.buffer.len(), 5);
        assert_eq!(eb.mode(), EventBufferMode::WaitForUserDial);
        assert_eq!(eb.as_str(), "12345");

        assert_eq!(
            true,
            eb.push(EventBufferMode::Dtmf, KeypadEvent::KeyPress('1'))
        );
        assert_eq!(eb.buffer.len(), 1);
        assert_eq!(eb.mode(), EventBufferMode::Dtmf);
        assert_eq!(eb.as_str(), "1");
    }

    #[test_case]
    fn wait_for_user_dials() {
        trace!("wait_for_user_dials");
        let mut eb = EventBuffer::new();

        let number = "22233334444";

        for c in number.chars() {
            assert_eq!(
                false,
                eb.push(EventBufferMode::WaitForUserDial, KeypadEvent::KeyPress(c))
            );
        }

        // Long presses are ignored execpt '#'
        assert_eq!(
            false,
            eb.push(
                EventBufferMode::WaitForUserDial,
                KeypadEvent::LongPress('1')
            )
        );

        assert_eq!(
            true,
            eb.push(
                EventBufferMode::WaitForUserDial,
                KeypadEvent::LongPress('#')
            )
        );
        assert_eq!(eb.as_str(), number);
    }
}
