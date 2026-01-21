OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* [修改] 從 0x80200000 改為 0x10000 */
    . = 0x10000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
    }
}