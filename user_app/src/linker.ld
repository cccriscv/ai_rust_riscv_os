/* user_app/linker.ld */
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* 必須是 0x80200000，不能是預設值 */
    . = 0x80200000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    /* ... 其他段 ... */
}