#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr;
                         // requires a debugger

use cortex_m_rt::entry;
use nucleo_f767zi::hal::stm32f7x7;

#[entry]
fn main() -> ! {
    let peripherals = stm32f7x7::Peripherals::take().unwrap();

    let _gpioa = &peripherals.GPIOA;

    loop {
        // your code goes here
    }
}
