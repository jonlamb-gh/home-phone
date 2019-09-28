use core::panic::PanicInfo;
use log::error;

#[panic_handler]
fn panic_handler_for_tests(info: &PanicInfo) -> ! {
    error!("{}", info);
    crate::exit(crate::ExitCode::Failed)
}
