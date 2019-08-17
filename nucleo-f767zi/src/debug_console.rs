use super::Serial3;
use crate::hal::prelude::*;
use crate::hal::serial::{Rx, Tx};
use crate::hal::stm32f7x7::USART3;

pub struct DebugConsole {
    tx: Tx<USART3>,
    _rx: Rx<USART3>,
}

impl DebugConsole {
    pub fn new(serial: Serial3) -> Self {
        let (tx, _rx) = serial.split();

        DebugConsole { tx, _rx }
    }
}

impl ::core::fmt::Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for &b in s.as_bytes() {
            block!(self.tx.write(b as _)).ok();
        }
        Ok(())
    }
}
