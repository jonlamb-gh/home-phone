#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use lib::hal::{delay::Delay, prelude::*, serial::config::Config, serial::Serial, stm32};
use lib::logger::Logger;
use log::{info, LevelFilter};

mod panic_handler;

static GLOBAL_LOGGER: Logger = Logger::new();

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

    let mut delay = Delay::new(cp.SYST, clocks);

    let gpiod = dp.GPIOD.split();
    let pin_tx = gpiod.pd8.into_alternate_af7();
    let pin_rx = gpiod.pd9.into_alternate_af7();

    let serial = Serial::usart3(
        dp.USART3,
        (pin_tx, pin_rx),
        Config {
            baudrate: 115_200.bps(),
            ..Default::default()
        },
        clocks,
    )
    .unwrap();

    let (tx, _rx) = serial.split();

    GLOBAL_LOGGER.set_inner(tx);

    log::set_logger(&GLOBAL_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    loop {
        info!("Hello world");
        delay.delay_ms(1000_u32);
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
