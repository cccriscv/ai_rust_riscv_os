.section .text.entry
    .globl _start
_start:
    # 設定堆疊指標，我們暫時隨便找個高位址
    # QEMU 預設有 128MB RAM，所以 0x81000000 是安全的
    la sp, _stack_top
    call rust_main

loop:
    j loop

    .section .bss
    .align 4
    .space 4096 * 4 # 16KB 的堆疊空間
_stack_top: