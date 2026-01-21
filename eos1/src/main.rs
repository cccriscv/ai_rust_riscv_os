#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;

use core::panic::PanicInfo;
use task::{Task, Context, STACK_SIZE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn trap_entry();
}

const CLINT_MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;
const CLINT_MTIME: *const u64 = 0x0200_BFF8 as *const u64;
// 設定時間間隔，太快的話 print 會一直被打斷，這裡設約 0.5 ~ 1 秒
const INTERVAL: u64 = 5_000_000; 

fn set_next_timer() {
    unsafe {
        let now = CLINT_MTIME.read_volatile();
        CLINT_MTIMECMP.write_volatile(now + INTERVAL);
    }
}

static mut TASK1: Task = Task::new();
static mut TASK2: Task = Task::new();
static mut CTX1: Context = Context::empty();
static mut CTX2: Context = Context::empty();
static mut CURR_TASK: usize = 1;

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }

    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    // Timer Interrupt
    if is_interrupt && code == 7 {
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
    } else {
        let mepc: usize;
        let mtval: usize;
        unsafe {
            core::arch::asm!("csrr {}, mepc", out(reg) mepc);
            core::arch::asm!("csrr {}, mtval", out(reg) mtval);
        }
        println!("\n[FATAL] Trap: mcause={}, mepc={:x}, mtval={:x}", mcause, mepc, mtval);
        loop {}
    }
}

// --- 任務邏輯修改 ---

fn task1() -> ! {
    let mut count = 0;
    loop {
        // 印出詳細資訊：計數器 與 堆疊變數位址 (證明堆疊隔離)
        println!("Task 1: count = {}, addr = {:p}", count, &count);
        count += 1;
        
        // 延遲迴圈 (避免跑太快洗版)
        for _ in 0..5000000 {}
    }
}

fn task2() -> ! {
    let mut count = 0;
    loop {
        println!("Task 2: count = {}, addr = {:p}", count, &count);
        count += 1;
        
        for _ in 0..5000000 {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS Preemptive (M-Mode Only)");
    println!("-----------------------------------");

    unsafe {
        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        CTX1.regs[2] = stack1_top as u64; 
        CTX1.mepc = task1 as u64;

        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        CTX2.regs[2] = stack2_top as u64;
        CTX2.mepc = task2 as u64;

        core::arch::asm!("csrw mtvec, {}", in(reg) trap_entry);
        core::arch::asm!("csrw mscratch, {}", in(reg) &raw mut CTX1);

        // 設定 mstatus (回到 Machine Mode)
        // MPP (11:12) = 11 -> Machine Mode
        // MPIE (7) = 1    -> 開中斷
        // FS (13:14) = 01 -> 開 FPU (避免 println 優化報錯)
        let mstatus: usize = (3 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);

        set_next_timer();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) 1 << 7);

        println!("[OS] Scheduler started...");

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