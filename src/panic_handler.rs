use core::fmt::Write;
use core::intrinsics;
use core::panic::PanicInfo;
use lib::hal::bcm2711::gpio::GPIO;
use lib::hal::bcm2711::mbox::MBOX;
use lib::hal::bcm2711::uart1::UART1;
use lib::hal::clocks::Clocks;
use lib::hal::mailbox::Mailbox;
use lib::hal::prelude::*;
use lib::hal::serial::Serial;
use lib::hal::time::Bps;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    if let Ok(clocks) = Clocks::freeze(&mut mbox) {
        let gpio = GPIO::new();

        let gp = gpio.split();
        let tx = gp.p14.into_alternate_af5();
        let rx = gp.p15.into_alternate_af5();

        let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);
        writeln!(serial, "\n\n{}\n\n", info).ok();
    }

    unsafe { intrinsics::abort() }
}
