use core::intrinsics;
use core::panic::PanicInfo;

// TODO panic w/uart3 output
//
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    //let mut mbox = Mailbox::new(MBOX::new());
    //if let Ok(clocks) = Clocks::freeze(&mut mbox) {
    //    let gpio = GPIO::new();
    //
    //    let gp = gpio.split();
    //    let tx = gp.p14.into_alternate_af5();
    //    let rx = gp.p15.into_alternate_af5();
    //
    //    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200),
    // clocks);    writeln!(serial, "\n\n{}\n\n", info).ok();
    //}

    unsafe { intrinsics::abort() }
}
