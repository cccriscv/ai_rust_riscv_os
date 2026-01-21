OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* QEMU RISC-V virt 機器從 0x80000000 開始 */
    . = 0x80000000;

    .text : {
        *(.text.entry) /* 確保啟動程式碼放在最前面 */
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

    /DISCARD/ : {
        *(.eh_frame)
    }
}