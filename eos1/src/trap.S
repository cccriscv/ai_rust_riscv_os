.section .text
# 確保 trap handler 不使用壓縮指令，保持對齊
.option norvc

# ============================================
# 1. 巨集定義 (必須放在最前面)
# ============================================

.macro SAVE_CONTEXT
    # 交換 sp 和 mscratch
    # 如果是從 User Mode 進來，mscratch 是 Kernel Stack，sp 是 User Stack
    # 交換後 sp 指向 Kernel Stack
    csrrw sp, mscratch, sp
    
    # 保存暫存器到 Kernel Stack
    sd x1, 1*8(sp)
    
    # 保存原本的 sp (現在在 mscratch 裡)
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
    
    # 保存發生中斷的位址 (mepc)
    csrr t0, mepc
    sd t0, 32*8(sp)
.endm

.macro RESTORE_CONTEXT
    # 恢復 mepc
    ld t0, 32*8(sp)
    csrw mepc, t0
    
    # 恢復一般暫存器
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
    
    # 恢復原本的 sp (User Stack Pointer)
    # 此時 sp 指向 Context，Context 的 offset 2 存著原本的 sp
    ld t0, 2*8(sp)
    # 把 User SP 暫存到 mscratch
    csrw mscratch, t0
    
    # 恢復 t0 (x5)
    ld x5, 5*8(sp)
    
    # 最後交換 sp 和 mscratch
    # sp 變回 User Stack，mscratch 變回 Kernel Stack
    csrrw sp, mscratch, sp
    
    mret
.endm

# ============================================
# 2. 中斷入口點 (Direct Mode)
# ============================================

.globl trap_vector
.align 4
trap_vector:
    # 1. 保存現場
    SAVE_CONTEXT

    # 2. 準備呼叫 Rust 函式
    # s0 = 當前的 Context 指標 (原本的 sp)
    mv s0, sp
    
    # 3. 切換到核心專用的 Trap Stack (避免 Stack Overflow)
    la sp, _trap_stack_top

    # 4. 傳遞參數並呼叫 handle_trap
    # fn handle_trap(ctx: *mut Context) -> *mut Context
    # a0 = s0 (Context 指標)
    mv a0, s0
    call handle_trap

    # 5. 恢復 Stack 指標
    # handle_trap 回傳的可能是新的 Context 指標 (如果是排程切換)
    # 所以我們把 a0 (回傳值) 移回 sp
    mv sp, a0

    # 6. 恢復現場
    RESTORE_CONTEXT

# ============================================
# 3. Trap Stack (BSS Section)
# ============================================
.section .bss
.align 16
_trap_stack:
    .space 4096 * 4
_trap_stack_top: