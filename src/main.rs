#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr;
                         // requires a debugger

use cortex_m_rt::entry;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::led::Leds;

#[entry]
fn main() -> ! {
    let mut core_peripherals =
        cortex_m::Peripherals::take().expect("Failed to take cortex_m::Peripherals");
    let peripherals =
        stm32f7x7::Peripherals::take().expect("Failed to take stm32f7x7::Peripherals");

    core_peripherals.SCB.enable_icache();
    core_peripherals
        .SCB
        .enable_dcache(&mut core_peripherals.CPUID);

    let mut flash = peripherals.FLASH.constrain();
    let mut rcc = peripherals.RCC.constrain();

    let mut gpiob = peripherals.GPIOB.split(&mut rcc.ahb1);

    let led_r = gpiob
        .pb14
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_g = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_b = gpiob
        .pb7
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    // Default clock configuration runs at 16 MHz
    let _clocks = rcc.cfgr.freeze(&mut flash.acr);

    // TODO
    // configure maximum clock frequency at 200 MHz
    //let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

    let mut leds = Leds::new(led_r, led_g, led_b);
    for led in leds.iter_mut() {
        led.on();
        //led.off();
    }

    loop {
        // your code goes here
    }
}
