use core::fmt::Write;
use crate::build_info;

pub fn build_info<T: Write>(writer: &mut T) {
    // Read the device unique ID, see 45.6
    let id1: u32 = unsafe { *(0x1FF0_F420 as *const u32) };
    let id2: u32 = unsafe { *(0x1FF0_F424 as *const u32) };
    let id3: u32 = unsafe { *(0x1FF0_F428 as *const u32) };

    write!(
        writer,
        "Version: {} {}\r\nBuilt: {}\r\nCompiler: {}\r\nMCU ID: {:08X}{:08X}{:08X}\r\n",
        build_info::PKG_VERSION,
        build_info::GIT_VERSION.unwrap(),
        build_info::BUILT_TIME_UTC,
        build_info::RUSTC_VERSION,
        id3,
        id2,
        id1
    ).ok();
}
