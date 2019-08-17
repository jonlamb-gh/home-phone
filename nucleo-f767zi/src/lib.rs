#![no_std]

#[macro_use(block)]
extern crate nb;

pub extern crate stm32f767_hal as hal;

use crate::hal::gpio::gpioa::PA3;
use crate::hal::gpio::gpiob::PB1;
// use crate::hal::gpio::gpioc::{PC0, PC13, PC2, PC3};
use crate::hal::gpio::gpiod::{PD8, PD9};
// use crate::hal::gpio::gpiof::{PF10, PF3, PF5};
// use crate::hal::gpio::{Analog, Input, PullDown, AF7};
use crate::hal::gpio::{Analog, Input, AF7};
use crate::hal::serial::Serial;
use crate::hal::stm32f7x7::USART3;

pub mod debug_console;
pub mod led;

// On board USER button connected via PC13
// pub type UserButtonPin = PC13<Input<PullDown>>;

// On board stlink/console port connected to USART3 via PD8 and PD9 pins
pub type Serial3 = Serial<USART3, (PD8<AF7>, PD9<AF7>)>;

// Analog inputs labeled A0:A5 on CN9, A6:A7 on CN10
// PA3, ADC123_IN3
pub type AnalogInput0Pin = PA3<Input<Analog>>;
// PC0, ADC123_IN10
// pub type AnalogInput1Pin = PC0<Input<Analog>>;
// PC3, ADC123_IN13
// pub type AnalogInput2Pin = PC3<Input<Analog>>;
// PF3, ADC3_IN9
// pub type AnalogInput3Pin = PF3<Input<Analog>>;
// PF5, ADC3_IN15
// pub type AnalogInput4Pin = PF5<Input<Analog>>;
// PF10, ADC3_IN8
// pub type AnalogInput5Pin = PF10<Input<Analog>>;
// PB1, ADC12_IN9
pub type AnalogInput6Pin = PB1<Input<Analog>>;
// PC2, ADC123_IN12
// pub type AnalogInput7Pin = PC2<Input<Analog>>;
