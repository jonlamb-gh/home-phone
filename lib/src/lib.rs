#![no_std]

pub extern crate stm32f4xx_hal as hal;

pub mod display;
//pub mod keypad;
pub mod logger;
pub mod net;
pub mod phone_number;
pub mod phone_state;
pub mod rtc;
pub mod sync;
