#![no_std]
#![no_main]
// [修正 1] 刪除 #![feature(alloc_error_handler)]，因為在較新的 Rust 這已經穩定了

#[macro_use]
mod uart;
mod task;
mod heap;

extern crate alloc;
use alloc::vec::Vec;
use alloc::boxed::Box;
// [修正 2] 移除未使用的 String import，避免警告
// use alloc::string::String; 
use alloc::format;

use core::panic::PanicInfo;
use core::fmt;
use task::{Task, Context, STACK_SIZE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn trap_entry();
}

const CLINT_MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;
const CLINT_MTIME: *const u64 = 0x0200_BFF8 as *const u64;
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

const SYSCALL_PUTCHAR: u64 = 1;

fn sys_putchar(c: u8) {
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_PUTCHAR,
            in("a0") c,
        );
    }
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

fn task1() -> ! {
    let mut count = 0;
    let heap_val = Box::new(999);
    let mut history = Vec::new();

    loop {
        if history.len() >= 5 {
            history.remove(0);
        }
        history.push(count);

        let msg = format!("Task 1 [Heap]: Box={}, Hist={:?}", *heap_val, history);
        user_println!("{}", msg);
        
        count += 1;
        for _ in 0..5000000 {}
    }
}

fn task2() -> ! {
    let mut count = 0;
    loop {
        user_println!("Task 2 [User]: count = {}", count);
        count += 1;
        for _ in 0..5000000 {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }

    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
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
        if code == 8 {
            unsafe {
                let id = (*ctx_ptr).regs[17];
                let arg0 = (*ctx_ptr).regs[10];

                match id {
                    SYSCALL_PUTCHAR => {
                        print!("{}", arg0 as u8 as char);
                    }
                    _ => {
                        println!("Unknown Syscall: {}", id);
                    }
                }
                (*ctx_ptr).mepc += 4;
                return ctx_ptr;
            }
        }
    }

    let mepc: usize;
    let mtval: usize;
    unsafe {
        core::arch::asm!("csrr {}, mepc", out(reg) mepc);
        core::arch::asm!("csrr {}, mtval", out(reg) mtval);
    }
    println!("\n[FATAL] Trap: mcause={}, mepc={:x}, mtval={:x}", mcause, mepc, mtval);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS with Heap & User Mode       ");
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

        println!("[OS] Heap initialized. Jumping to User Mode...");

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