OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x80000000;

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

    /* [新增] 定義核心結束的位址符號 */
    /* 透過 ALIGN(4096) 確保它是從下一個 4KB 邊界開始 */
    . = ALIGN(4096);
    ekernel = .;
}