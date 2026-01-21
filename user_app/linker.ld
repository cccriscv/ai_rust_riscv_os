OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* [重點] 設定載入位址為 0x80200000 */
    . = 0x80200000;

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