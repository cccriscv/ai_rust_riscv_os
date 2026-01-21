#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;

use core::panic::PanicInfo;
use core::fmt; // 引入 fmt 以支援格式化輸出
use task::{Task, Context, STACK_SIZE};

// 引入彙編代碼
core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn trap_entry();
}

// --- 硬體常數 ---
const CLINT_MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;
const CLINT_MTIME: *const u64 = 0x0200_BFF8 as *const u64;
const INTERVAL: u64 = 5_000_000;

fn set_next_timer() {
    unsafe {
        let now = CLINT_MTIME.read_volatile();
        CLINT_MTIMECMP.write_volatile(now + INTERVAL);
    }
}

// --- 全域變數 ---
static mut TASK1: Task = Task::new();
static mut TASK2: Task = Task::new();
static mut CTX1: Context = Context::empty();
static mut CTX2: Context = Context::empty();
static mut CURR_TASK: usize = 1;

// --- [User Mode] System Call 實作 ---

const SYSCALL_PUTCHAR: u64 = 1;

/// 這是最底層的 System Call 觸發函式
/// 它執行 `ecall` 指令，將控制權交給 Kernel
fn sys_putchar(c: u8) {
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_PUTCHAR, // a7: 系統呼叫編號
            in("a0") c,               // a0: 參數
        );
    }
}

/// 為了支援 println! 的格式化功能，我們定義一個 UserOut 結構
struct UserOut;

/// 實作 fmt::Write Trait，讓它可以被 Rust 的格式化巨集使用
impl fmt::Write for UserOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            sys_putchar(c);
        }
        Ok(())
    }
}

/// 定義 User Mode 專用的 println! 巨集
#[macro_export]
macro_rules! user_println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        // 建立 UserOut 並寫入格式化字串
        let mut writer = UserOut;
        let _ = write!(writer, $($arg)*);
        let _ = write!(writer, "\n");
    });
}

// --- [User Mode] 任務邏輯 ---

fn task1() -> ! {
    let mut count = 0;
    loop {
        // 現在我們呼叫的是會觸發 ecall 的 user_println!
        // 而不是直接寫硬體的 println!
        user_println!("Task 1 [User]: count = {}, addr = {:p}", count, &count);
        count += 1;
        
        // 模擬運算延遲
        for _ in 0..5000000 {}
    }
}

fn task2() -> ! {
    let mut count = 0;
    loop {
        user_println!("Task 2 [User]: count = {}, addr = {:p}", count, &count);
        count += 1;
        
        for _ in 0..5000000 {}
    }
}

// --- [Kernel Mode] Trap Handler ---

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }

    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
        // [1] 處理 Timer 中斷 (搶佔式多工)
        if code == 7 {
            set_next_timer();
            unsafe {
                if CURR_TASK == 1 {
                    CURR_TASK = 2;
                    return &raw mut CTX2;
                } else {
                    CURR_TASK = 1;
                    return &raw mut CTX1;
                }
            }
        }
    } else {
        // [2] 處理 System Call (User Mode Ecall)
        if code == 8 {
            unsafe {
                // 從 Context 中讀取 User Mode 傳來的暫存器
                // a7 (x17) 是 ID, a0 (x10) 是參數
                let id = (*ctx_ptr).regs[17];
                let arg0 = (*ctx_ptr).regs[10];

                match id {
                    SYSCALL_PUTCHAR => {
                        // Kernel 代替 User 印出字元到 UART
                        print!("{}", arg0 as u8 as char);
                    }
                    _ => {
                        println!("Unknown Syscall: {}", id);
                    }
                }

                // [重要] 因為是 Exception，mepc 指向 ecall 指令
                // 我們必須手動 +4 跳過 ecall，否則返回後會變成無窮迴圈
                (*ctx_ptr).mepc += 4;
                
                // 處理完後，繼續執行同一個任務
                return ctx_ptr;
            }
        }
    }

    // [3] 處理其他異常 (Crash)
    let mepc: usize;
    let mtval: usize;
    unsafe {
        core::arch::asm!("csrr {}, mepc", out(reg) mepc);
        core::arch::asm!("csrr {}, mtval", out(reg) mtval);
    }
    println!("\n[FATAL] Trap: mcause={}, mepc={:x}, mtval={:x}", mcause, mepc, mtval);
    loop {}
}

// --- [Kernel Mode] 初始化 ---

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS with User Mode & Syscalls   ");
    println!("-----------------------------------");

    unsafe {
        // [1] 設定 PMP (Physical Memory Protection)
        // 允許 User Mode 存取所有記憶體 (RWX)
        // pmpaddr0 = -1 (All Memory), pmpcfg0 = 0x1F (NAPOT + RWX + Lock)
        core::arch::asm!("csrw pmpaddr0, {}", in(reg) !0usize);
        core::arch::asm!("csrw pmpcfg0, {}", in(reg) 0x1Fusize);

        // [2] 初始化 Context
        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        CTX1.regs[2] = stack1_top as u64; 
        CTX1.mepc = task1 as u64;

        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        CTX2.regs[2] = stack2_top as u64;
        CTX2.mepc = task2 as u64;

        // [3] 設定 Trap
        core::arch::asm!("csrw mtvec, {}", in(reg) trap_entry);
        core::arch::asm!("csrw mscratch, {}", in(reg) &raw mut CTX1);

        // [4] 設定 mstatus: 切換到 User Mode
        // MPP (11:12) = 00 -> User Mode
        // MPIE (7) = 1    -> 開啟中斷
        // FS (13:14) = 01 -> 開啟 FPU
        let mstatus: usize = (0 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);

        // [5] 啟動計時器
        set_next_timer();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) 1 << 7);

        println!("[OS] Switching to User Mode...");

        // [6] 跳轉進入 User Mode
        core::arch::asm!(
            "mv sp, {}",
            "csrw mepc, {}",
            "mret",
            in(reg) CTX1.regs[2],
            in(reg) CTX1.mepc
        );
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[PANIC] {}", info);
    loop {}
}