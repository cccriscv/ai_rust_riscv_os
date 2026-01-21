.section .text

# [關鍵修正 1] 禁止使用壓縮指令 (RVC)
# 確保每一條跳轉指令都佔用 4 bytes，維持向量表對齊
.option norvc

.align 2
.globl trap_vector
trap_vector:
    j exception_entry       # 0: User Software
    j exception_entry       # 1: Supervisor Software
    j exception_entry       # 2: Reserved
    j exception_entry       # 3: Machine Software
    j exception_entry       # 4: User Timer
    j exception_entry       # 5: Supervisor Timer
    j exception_entry       # 6: Reserved
    j timer_entry           # 7: Machine Timer
    j exception_entry       # 8: User External
    j exception_entry       # 9: Supervisor External
    j exception_entry       # 10: Reserved
    j external_entry        # 11: Machine External (PLIC)
    j exception_entry       # 12
    j exception_entry       # 13
    j exception_entry       # 14
    j exception_entry       # 15

# [關鍵修正 2] 恢復允許壓縮指令 (如果是 riscv64gc，這通常是預設值)
.option rvc

# --- 保存與恢復巨集 ---

.macro SAVE_CONTEXT
    csrrw sp, mscratch, sp
    sd x1, 1*8(sp)
    csrr t0, mscratch
    sd t0, 2*8(sp)
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    sd x5, 5*8(sp)
    sd x6, 6*8(sp)
    sd x7, 7*8(sp)
    sd x8, 8*8(sp)
    sd x9, 9*8(sp)
    sd x10, 10*8(sp)
    sd x11, 11*8(sp)
    sd x12, 12*8(sp)
    sd x13, 13*8(sp)
    sd x14, 14*8(sp)
    sd x15, 15*8(sp)
    sd x16, 16*8(sp)
    sd x17, 17*8(sp)
    sd x18, 18*8(sp)
    sd x19, 19*8(sp)
    sd x20, 20*8(sp)
    sd x21, 21*8(sp)
    sd x22, 22*8(sp)
    sd x23, 23*8(sp)
    sd x24, 24*8(sp)
    sd x25, 25*8(sp)
    sd x26, 26*8(sp)
    sd x27, 27*8(sp)
    sd x28, 28*8(sp)
    sd x29, 29*8(sp)
    sd x30, 30*8(sp)
    sd x31, 31*8(sp)
    csrr t0, mepc
    sd t0, 32*8(sp)
.endm

.macro RESTORE_CONTEXT
    ld t0, 32*8(sp)
    csrw mepc, t0
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    ld x6, 6*8(sp)
    ld x7, 7*8(sp)
    ld x8, 8*8(sp)
    ld x9, 9*8(sp)
    ld x10, 10*8(sp)
    ld x11, 11*8(sp)
    ld x12, 12*8(sp)
    ld x13, 13*8(sp)
    ld x14, 14*8(sp)
    ld x15, 15*8(sp)
    ld x16, 16*8(sp)
    ld x17, 17*8(sp)
    ld x18, 18*8(sp)
    ld x19, 19*8(sp)
    ld x20, 20*8(sp)
    ld x21, 21*8(sp)
    ld x22, 22*8(sp)
    ld x23, 23*8(sp)
    ld x24, 24*8(sp)
    ld x25, 25*8(sp)
    ld x26, 26*8(sp)
    ld x27, 27*8(sp)
    ld x28, 28*8(sp)
    ld x29, 29*8(sp)
    ld x30, 30*8(sp)
    ld x31, 31*8(sp)
    ld t0, 2*8(sp)
    csrw mscratch, t0
    ld x5, 5*8(sp)
    csrrw sp, mscratch, sp
    mret
.endm

# --- Handler Entries ---

.align 4
exception_entry:
    SAVE_CONTEXT
    mv s0, sp
    la sp, _trap_stack_top
    mv a0, s0
    call handle_trap
    mv sp, a0
    RESTORE_CONTEXT

.align 4
timer_entry:
    SAVE_CONTEXT
    mv s0, sp
    la sp, _trap_stack_top
    mv a0, s0
    call handle_timer
    mv sp, a0
    RESTORE_CONTEXT

.align 4
external_entry:
    SAVE_CONTEXT
    mv s0, sp
    la sp, _trap_stack_top
    mv a0, s0
    call handle_external
    mv sp, a0
    RESTORE_CONTEXT

.section .bss
.align 16
_trap_stack:
    .space 4096 * 4
_trap_stack_top: