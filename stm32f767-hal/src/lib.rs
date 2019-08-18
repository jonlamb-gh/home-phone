#![no_std]
#![deny(warnings)]

extern crate embedded_hal as hal;
pub extern crate oxcc_stm32f767 as stm32f7x7;

pub mod delay;
pub mod flash;
pub mod gpio;
pub mod iwdg;
pub mod prelude;
pub mod rcc;
pub mod serial;
pub mod time;
pub mod timer;
