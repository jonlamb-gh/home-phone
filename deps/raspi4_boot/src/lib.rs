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

/// Prepare and execute transition from EL2 to EL1.
#[inline]
fn setup_and_enter_el1_from_el2() -> ! {
    use cortex_a::{asm, regs::*};

    #[cfg(feature = "bare-metal-loadaddr")]
    const STACK_START: u64 = 0x80_000;
    #[cfg(feature = "uboot-loadaddr")]
    const STACK_START: u64 = 0x10_0000;

    // Enable timer counter registers for EL1
    CNTHCTL_EL2.write(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);

    // No offset for reading the counters
    CNTVOFF_EL2.set(0);

    // Set EL1 execution state to AArch64
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);

    // Set up a simulated exception return.
    //
    // First, fake a saved program status, where all interrupts were
    // masked and SP_EL1 was used as a stack pointer.
    SPSR_EL2.write(
        SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked
            + SPSR_EL2::M::EL1h,
    );

    // Second, let the link register point to reset().
    ELR_EL2.set(reset as *const () as u64);

    // Set up SP_EL1 (stack pointer), which will be used by EL1 once
    // we "return" to it.
    SP_EL1.set(STACK_START);

    // Use `eret` to "return" to EL1. This will result in execution of
    // `reset()` in EL1.
    asm::eret()
}

/// Entrypoint of the processor.
///
/// Parks all cores except core0 and checks if we started in EL2. If
/// so, proceeds with setting up EL1.
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
    const EL2: u32 = CurrentEL::EL::EL2.value;

    if (CORE_0 == MPIDR_EL1.get() & CORE_MASK) && (EL2 == CurrentEL.get()) {
        setup_and_enter_el1_from_el2()
    }

    // if not core0 or EL != 2, infinitely wait for events
    loop {
        asm::wfe();
    }
}
