#![no_std]
#![no_main]
#![cfg_attr(not(test), feature(core_intrinsics))]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(test_runner::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

#[cfg(not(test))]
mod panic_handler;

use lib::hal::bcm2711::gpio::GPIO;
use lib::hal::bcm2711::mbox::MBOX;
use lib::hal::bcm2711::sys_timer::SysTimer;
use lib::hal::bcm2711::uart1::UART1;
use lib::hal::clocks::Clocks;
use lib::hal::mailbox::*;
use lib::hal::prelude::*;
use lib::hal::serial::Serial;
use lib::hal::time::Bps;
use lib::logger::Logger;
use log::{info, LevelFilter};

static GLOBAL_LOGGER: Logger = Logger::new();

#[cfg(not(test))]
raspi3_boot::entry!(main);
fn main() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    GLOBAL_LOGGER.set_inner(serial);
    log::set_logger(&GLOBAL_LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap();

    let sys_timer = SysTimer::new();
    let mut sys_counter = sys_timer.split().sys_counter;

    info!("Starting");

    loop {
        sys_counter.delay_ms(500u32);
    }
}

#[cfg(test)]
mod tests {
    // Uses the QEMU panic handler
    use super::*;
    use log::trace;

    // TODO - move this into the test-runner crate?
    raspi3_boot::entry!(test_entry);
    pub fn test_entry() -> ! {
        let mut mbox = Mailbox::new(MBOX::new());
        let clocks = Clocks::freeze(&mut mbox).unwrap();
        let gpio = GPIO::new();
        let gp = gpio.split();

        let tx = gp.p14.into_alternate_af5();
        let rx = gp.p15.into_alternate_af5();

        let serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

        GLOBAL_LOGGER.set_inner(serial);
        log::set_logger(&GLOBAL_LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();

        crate::test_main();

        qemu::exit(qemu::ExitCode::Success)
    }

    #[test_case]
    fn it_works() {
        trace!("it_works");
        assert_eq!(2 + 2, 4);
    }
}
