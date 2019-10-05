/* Defined in the raspi3_boot crate */
ENTRY(_boot_cores);

SECTIONS
{
    /* Bare metal */
    /* . = 0x80000; */

    /* When loading with u-boot, put it somewhere else as to not overwrite u-boot */
    . = 0x0100000;

    .text :
    {
        KEEP(*(.text.boot)) *(.text .text.*)
    }

    .rodata ALIGN(16):
    {
        *(.rodata .rodata.*)
    }

    .data ALIGN(16):
    {
        *(.data .data.*)
    }

    .bss ALIGN(16):
    {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
