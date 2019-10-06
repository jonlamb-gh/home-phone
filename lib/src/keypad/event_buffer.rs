use crate::keypad::KeypadEvent;
use crate::phone_number::MAX_DIGITS;
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
                    self.buffer.push(c).unwrap_or(debug!("EventBuffer full"));
                    false
                }
                KeypadEvent::LongPress(c) => c == '#',
            },
            EventBufferMode::Dtmf => {
                if let KeypadEvent::KeyPress(c) = event {
                    self.buffer.push(c).unwrap_or(debug!("EventBuffer full"));
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
    use log::{debug, trace};

    #[test_case]
    fn todo() {
        trace!("todo");
        debug!("todo");
        let mut eb = EventBuffer::new();
    }
}
