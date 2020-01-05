use crate::hal::{prelude::*, serial::Tx, stm32::USART3};
use crate::sync::SingleCoreLock;
use core::fmt::Write;
use log::{Metadata, Record};

type Inner = Tx<USART3>;

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
