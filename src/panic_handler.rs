use core::intrinsics;
use core::panic::PanicInfo;
use log::error;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    error!("{}", info);
    unsafe { intrinsics::abort() }
}
