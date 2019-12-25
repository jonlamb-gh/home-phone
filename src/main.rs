#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use lib::hal::{prelude::*, serial::config::Config, serial::Serial, stm32};

mod panic_handler;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    loop {
        // TODO
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
