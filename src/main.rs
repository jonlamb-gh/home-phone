#![no_std]
#![no_main]

// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                         // extern crate panic_abort; // requires nightly
                         // extern crate panic_itm; // logs messages over ITM; requires ITM support
                         // extern crate panic_semihosting; // logs messages to the host stderr;
                         // requires a debugger

mod board_info;

use core::fmt::Write;
use cortex_m_rt::entry;
use nucleo_f767zi::debug_console::DebugConsole;
use nucleo_f767zi::hal::prelude::*;
use nucleo_f767zi::hal::serial::Serial;
use nucleo_f767zi::hal::stm32f7x7;
use nucleo_f767zi::hal::timer::Timer;
use nucleo_f767zi::led::{Color, Leds};

// Pull in build information (from `built` crate)
mod build_info {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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
    let mut gpiod = peripherals.GPIOD.split(&mut rcc.ahb1);

    let led_r = gpiob
        .pb14
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_g = gpiob
        .pb0
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let led_b = gpiob
        .pb7
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let usart3_tx = gpiod.pd8.into_af7(&mut gpiod.moder, &mut gpiod.afrh);
    let usart3_rx = gpiod.pd9.into_af7(&mut gpiod.moder, &mut gpiod.afrh);

    // TODO - revist my RCC bits
    // Default clock configuration runs at 16 MHz
    //let _clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc.cfgr.freeze_max(&mut flash.acr);

    let mut leds = Leds::new(led_r, led_g, led_b);
    for led in leds.iter_mut() {
        //led.on();
        led.off();
    }

    // USART3 is routed up to the same USB port as the stlink
    let serial = Serial::usart3(
        peripherals.USART3,
        (usart3_tx, usart3_rx),
        115_200.bps(),
        clocks,
        &mut rcc.apb1,
    );

    let mut debug_console = DebugConsole::new(serial);

    let mut timer = Timer::tim2(peripherals.TIM2, 1.hz(), clocks, &mut rcc.apb1);

    writeln!(debug_console, "Board Initialized").ok();
    board_info::build_info(&mut debug_console);

    leds[Color::Green].toggle();

    loop {
        if timer.wait().is_ok() == true {
            leds[Color::Blue].toggle();
            leds[Color::Green].toggle();
            writeln!(debug_console, "*").ok();
        }
    }
}
