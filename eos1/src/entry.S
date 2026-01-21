.section .text.entry
    .globl _start
_start:
    # 設定堆疊指標
    la sp, _stack_top
    call rust_main

loop:
    j loop

    .section .bss
    .align 4
    # [修正] 加大堆疊空間防止 Stack Overflow
    # 4096 * 32 = 128KB (足夠安全了)
    .space 4096 * 32 
_stack_top: