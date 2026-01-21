#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;
mod heap;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use core::panic::PanicInfo;
use core::fmt;
use task::{Task, Context, STACK_SIZE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn trap_entry();
}

// --- 硬體與 Timer ---
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

// --- User Mode Syscall 介面 ---
const SYSCALL_PUTCHAR: u64 = 1;
const SYSCALL_GETCHAR: u64 = 2; // 新增 Syscall ID

fn sys_putchar(c: u8) {
    unsafe {
        core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c);
    }
}

/// 嘗試讀取一個字元，如果沒按鍵則回傳 0
fn sys_getchar() -> u8 {
    let mut ret: usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_GETCHAR,
            lateout("a0") ret, // 讀取回傳值
        );
    }
    ret as u8
}

struct UserOut;
impl fmt::Write for UserOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            sys_putchar(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! user_println {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = UserOut;
        let _ = write!(writer, $($arg)*);
        let _ = write!(writer, "\n");
    });
}

#[macro_export]
macro_rules! user_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = UserOut;
        let _ = write!(writer, $($arg)*);
    });
}

// --- [Shell Task] ---

fn task1() -> ! {
    user_println!("Shell initialized. Type 'help' for commands.");
    
    let mut command = String::new();
    user_print!("eos> "); // 提示符號

    loop {
        // 1. 嘗試讀取鍵盤
        let c = sys_getchar();
        
        if c != 0 {
            // 2. 處理 Enter 鍵 (CR=13 或 LF=10)
            if c == 13 || c == 10 {
                user_println!(""); // 換行
                
                // 3. 執行指令
                match command.as_str() {
                    "help" => {
                        user_println!("Available commands:");
                        user_println!("  help  - Show this message");
                        user_println!("  hello - Say hello");
                        user_println!("  clear - Clear command buffer");
                        user_println!("  panic - Test kernel panic");
                    },
                    "hello" => user_println!("Hello from User Mode!"),
                    "clear" => user_println!("Buffer cleared."),
                    "panic" => {
                        user_println!("Attempting to crash...");
                        unsafe {
                            // 使用 write_volatile 強制 CPU 執行該指令
                            // 繞過 Rust 的軟體 Null Check
                            (0x0 as *mut u8).write_volatile(0);
                        }
                    },
                    "" => {}, // 空指令
                    _ => user_println!("Unknown command: '{}'", command),
                }
                
                // 4. 重置 Buffer 並印出新的提示符
                command.clear();
                user_print!("eos> ");
            } 
            // 處理 Backspace (127 或 8)
            else if c == 127 || c == 8 {
                if !command.is_empty() {
                    command.pop();
                    // 簡易的倒退刪除效果 (BS + Space + BS)
                    sys_putchar(8);
                    sys_putchar(b' ');
                    sys_putchar(8);
                }
            }
            // 處理一般字元
            else {
                sys_putchar(c); // 回顯 (Echo) 到螢幕
                command.push(c as char);
            }
        }

        // 稍微讓出 CPU，避免佔用太多資源 (Polling 模式的缺點)
        // 在真實 OS 會用 Wait For Interrupt (WFI)
        for _ in 0..1000 {} 
    }
}

// Task 2 保持安靜，證明多工還在跑
fn task2() -> ! {
    let mut count = 0;
    loop {
        // 每隔很久印一次，避免干擾 Shell 輸入
        if count % 50 == 0 {
            // user_println!("[Background Task] I am still running...");
        }
        count += 1;
        for _ in 0..1000000 {}
    }
}

// --- Kernel Trap Handler ---

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }

    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
        if code == 7 { // Timer Interrupt
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
        // Exception & Syscall
        if code == 8 { // Ecall
            unsafe {
                let id = (*ctx_ptr).regs[17];
                let arg0 = (*ctx_ptr).regs[10];

                match id {
                    SYSCALL_PUTCHAR => {
                        print!("{}", arg0 as u8 as char);
                    }
                    SYSCALL_GETCHAR => {
                        // 呼叫 Driver 讀取
                        if let Some(c) = uart::_getchar() {
                            (*ctx_ptr).regs[10] = c as u64; // 回傳值放在 a0
                        } else {
                            (*ctx_ptr).regs[10] = 0; // 沒讀到回傳 0
                        }
                    }
                    _ => println!("Unknown Syscall: {}", id),
                }
                (*ctx_ptr).mepc += 4;
                return ctx_ptr;
            }
        } else {
            // 捕捉非法存取 (User Mode Crash)
            let mepc: usize;
            let mtval: usize;
            unsafe {
                core::arch::asm!("csrr {}, mepc", out(reg) mepc);
                core::arch::asm!("csrr {}, mtval", out(reg) mtval);
            }
            println!("\n[Segmentation Fault] App crashed at {:x}, accessing {:x}", mepc, mtval);
            println!("Killing task and rebooting shell...");
            
            // 簡單處置：重置目前任務的 PC 回到 task1 開頭 (復活術)
            unsafe {
                (*ctx_ptr).mepc = task1 as u64;
            }
            return ctx_ptr;
        }
    }

    let mepc: usize;
    unsafe { core::arch::asm!("csrr {}, mepc", out(reg) mepc); }
    println!("\n[FATAL] Trap: mcause={}, mepc={:x}", mcause, mepc);
    loop {}
}

// --- Kernel Entry ---

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS Interactive Shell           ");
    println!("-----------------------------------");

    unsafe {
        core::arch::asm!("csrw pmpaddr0, {}", in(reg) !0usize);
        core::arch::asm!("csrw pmpcfg0, {}", in(reg) 0x1Fusize);

        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        CTX1.regs[2] = stack1_top as u64; 
        CTX1.mepc = task1 as u64;

        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        CTX2.regs[2] = stack2_top as u64;
        CTX2.mepc = task2 as u64;

        core::arch::asm!("csrw mtvec, {}", in(reg) trap_entry);
        core::arch::asm!("csrw mscratch, {}", in(reg) &raw mut CTX1);

        let mstatus: usize = (0 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);

        set_next_timer();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) 1 << 7);

        println!("[OS] User Mode initialized.");

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