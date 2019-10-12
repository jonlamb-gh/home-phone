//! Low-level boot of the Raspberry's processor

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

/// Type check the user-supplied entry function.
#[macro_export]
macro_rules! entry {
    ($path:path) => {
        /// # Safety
        ///
        /// - User must ensure to provide a suitable main function for the platform.
        #[export_name = "main"]
        pub unsafe fn __main() -> ! {
            // type check the given path
            let f: fn() -> ! = $path;

            f()
        }
    };
}

/// Reset function.
///
/// Initializes the bss section before calling into the user's `main()`.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn reset() -> ! {
    extern "C" {
        // Boundaries of the .bss section, provided by the linker script
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }

    // Zeroes the .bss section
    r0::zero_bss(&mut __bss_start, &mut __bss_end);

    extern "Rust" {
        fn main() -> !;
    }

    main()
}

/// Entrypoint of the processor.
///
/// Parks all cores except core0, and then jumps to the internal
/// `reset()` function.
///
/// # Safety
///
/// - Linker script must ensure to place this function at `STACK_START`.
#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn _boot_cores() -> ! {
    use cortex_a::{asm, regs::*};

    const CORE_0: u64 = 0;
    const CORE_MASK: u64 = 0x3;

    #[cfg(feature = "bare-metal-loadaddr")]
    const STACK_START: u64 = 0x80_000;
    #[cfg(feature = "uboot-loadaddr")]
    const STACK_START: u64 = 0x10_0000;

    if CORE_0 == MPIDR_EL1.get() & CORE_MASK {
        SP.set(STACK_START);
        reset()
    } else {
        // if not core0, infinitely wait for events
        loop {
            asm::wfe();
        }
    }
}
