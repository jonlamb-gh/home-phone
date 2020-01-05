#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::cell::Cell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use lib::hal::prelude::*;
use lib::hal::serial::{config::Config, Serial};
use lib::hal::stm32;
use lib::logger::Logger;
use lib::sys_clock::SysClock;
use log::{info, LevelFilter};

mod panic_handler;

static GLOBAL_LOGGER: Logger = Logger::new();

static GLOBAL_SYST_MS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

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

    let mut sys_clock = SysClock::new(cp.SYST, clocks);

    let mut last_sec = 0;
    loop {
        let ms: u64 = cortex_m::interrupt::free(|cs| GLOBAL_SYST_MS.borrow(cs).get());
        sys_clock.set_time(ms);
        let time = sys_clock.now();

        let sec = time.as_secs();
        if sec != last_sec {
            info!("{}", lib::time::DisplayableInstant::from(time));
            last_sec = sec;
        }
    }
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let cell = GLOBAL_SYST_MS.borrow(cs);
        let t = cell.get();
        cell.replace(t.wrapping_add(1));
    })
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
