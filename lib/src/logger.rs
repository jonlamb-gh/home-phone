// TODO
// - expose this in deps/libs so the test framework uses it?

use crate::hal::bcm2711::uart1::UART1;
use crate::hal::gpio::{Alternate, Pin14, Pin15, AF5};
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::sync::SingleCoreLock;
use core::fmt::Write;
use log::{Metadata, Record};

type Inner = Serial<UART1, (Pin14<Alternate<AF5>>, Pin15<Alternate<AF5>>)>;

// TODO
// - make generic over UART0/1, requires Send for Pins/etc
pub struct Logger {
    inner: SingleCoreLock<Option<Inner>>,
}

impl Logger {
    pub const fn new() -> Self {
        Logger {
            inner: SingleCoreLock::new(None),
        }
    }

    pub fn set_inner(&self, inner: Inner) {
        self.inner.lock(|i| {
            let _ = i.replace(inner);
        });
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO
        true
    }

    fn log(&self, record: &Record) {
        self.inner.lock(|maybe_serial| {
            if let Some(serial) = maybe_serial {
                if self.enabled(record.metadata()) {
                    writeln!(serial, "[{}] {}", record.level(), record.args()).unwrap();
                }
            } else {
                panic!("Logger was used before being given its inner type");
            }
        });
    }

    fn flush(&self) {
        self.inner.lock(|maybe_serial| {
            if let Some(serial) = maybe_serial {
                serial.flush().unwrap();
            } else {
                panic!("Logger was used before being given its inner type");
            }
        });
    }
}
