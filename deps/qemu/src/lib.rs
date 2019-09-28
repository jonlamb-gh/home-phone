//! The asm lifted from:
//! https://github.com/cirosantilli/linux-kernel-module-cheat/blob/c53ccb02782e6b5ba94c38c72597101cde86c4ff/baremetal/arch/aarch64/semihost_exit.S

#![no_std]
#![feature(asm, core_intrinsics)]

mod panic_handler;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(usize)]
pub enum ExitCode {
    Success = 0,
    Failed = 1,
}

/// Make QEMU exit
///
/// NOTE: requires QEMU option `-semihosting`
pub fn exit(exit_code: ExitCode) -> ! {
    unsafe {
        asm!(
            "
            /* 0x20026 == ADP_Stopped_ApplicationExit */
            mov x1, #0x26
            movk x1, #2, lsl #16
            str x1, [sp,#0]

            /* Exit status code. Host QEMU process exits with that status. */
            mov x0, $0
            str x0, [sp,#8]

            /* x1 contains the address of parameter block.
             * Any memory address could be used. */
            mov x1, sp

            /* SYS_EXIT */
            mov w0, #0x18

            /* Do the semihosting call on A64. */
            hlt 0xf000
            "
            : // No output operands
            : "r" (exit_code)
            : "x0", "x1"
            : // No options
        );

        core::intrinsics::unreachable();
    }
}
