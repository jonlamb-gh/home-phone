#![no_std]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(test_runner::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

pub extern crate bcm2711_hal as hal;

pub mod logger;
pub mod net;
pub mod phone_number;
pub mod sync;

#[cfg(test)]
mod tests {
    use super::*;
    use hal::bcm2711::gpio::GPIO;
    use hal::bcm2711::mbox::MBOX;
    use hal::bcm2711::uart1::UART1;
    use hal::clocks::Clocks;
    use hal::mailbox::*;
    use hal::prelude::*;
    use hal::serial::Serial;
    use hal::time::Bps;
    use log::{trace, LevelFilter};
    use logger::Logger;

    static GLOBAL_LOGGER: Logger = Logger::new();

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
