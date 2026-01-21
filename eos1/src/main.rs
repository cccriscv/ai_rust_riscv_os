#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;
mod heap;
mod fs;
mod elf;
mod mm;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use core::panic::PanicInfo;
use core::fmt;
use task::{Task, Context, STACK_SIZE};
#[allow(unused_imports)]
use crate::mm::page_table::{PageTable, PTE_R, PTE_W, PTE_X, PTE_U, KERNEL_PAGE_TABLE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" {
    fn trap_vector();
}

// --- 硬體常數: CLINT (Timer) ---
const CLINT_MTIMECMP: *mut u64 = 0x0200_4000 as *mut u64;
const CLINT_MTIME: *const u64 = 0x0200_BFF8 as *const u64;
const INTERVAL: u64 = 5_000_000;

// --- 硬體常數: PLIC (Interrupt Controller) ---
const PLIC_BASE: usize = 0x0c00_0000;
const PLIC_PRIORITY: *mut u32 = PLIC_BASE as *mut u32;
const PLIC_ENABLE: *mut u32 = (PLIC_BASE + 0x2000) as *mut u32;
const PLIC_THRESHOLD: *mut u32 = (PLIC_BASE + 0x200000) as *mut u32;
const PLIC_CLAIM: *mut u32 = (PLIC_BASE + 0x200004) as *mut u32;

// --- 全域變數 ---
static mut TASK1: Task = Task::new();
static mut TASK2: Task = Task::new();
static mut CTX1: Context = Context::empty();
static mut CTX2: Context = Context::empty();
static mut CURR_TASK: usize = 1;

// --- 鍵盤緩衝區 (Ring Buffer) ---
// [修正 1] 定義常數，避免對 static mut 使用 .len()
const KEY_BUFFER_SIZE: usize = 256;
static mut KEY_BUFFER: [u8; KEY_BUFFER_SIZE] = [0; KEY_BUFFER_SIZE];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;

fn push_key(c: u8) {
    unsafe {
        // [修正 1] 使用常數 KEY_BUFFER_SIZE
        let next = (KEY_HEAD + 1) % KEY_BUFFER_SIZE;
        if next != KEY_TAIL { 
            KEY_BUFFER[KEY_HEAD] = c;
            KEY_HEAD = next;
        }
    }
}

fn pop_key() -> Option<u8> {
    unsafe {
        if KEY_HEAD == KEY_TAIL { return None; }
        let c = KEY_BUFFER[KEY_TAIL];
        // [修正 1] 使用常數 KEY_BUFFER_SIZE
        KEY_TAIL = (KEY_TAIL + 1) % KEY_BUFFER_SIZE;
        Some(c)
    }
}

// --- 初始化函式 ---

fn set_next_timer() {
    unsafe {
        let now = CLINT_MTIME.read_volatile();
        CLINT_MTIMECMP.write_volatile(now + INTERVAL);
    }
}

fn init_plic() {
    unsafe {
        // [修正 2] 使用原生指標呼叫 enable_interrupt，避免建立引用
        let writer = &raw mut uart::WRITER;
        (*writer).enable_interrupt();

        let irq_uart = 10; 

        PLIC_PRIORITY.add(irq_uart).write_volatile(1);
        PLIC_ENABLE.write_volatile(1 << irq_uart);
        PLIC_THRESHOLD.write_volatile(0);
    }
}

// --- Syscall ID & Wrappers ---
const SYSCALL_PUTCHAR: u64 = 1;
const SYSCALL_GETCHAR: u64 = 2;
const SYSCALL_FILE_LEN: u64 = 3;
const SYSCALL_FILE_READ: u64 = 4;
const SYSCALL_FILE_LIST: u64 = 5;
const SYSCALL_EXEC: u64 = 6;
const SYSCALL_EXIT: u64 = 93;

fn sys_putchar(c: u8) { unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c); } }
fn sys_getchar() -> u8 { let mut ret: usize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_GETCHAR, lateout("a0") ret); } ret as u8 }
fn sys_file_len(name: &str) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LEN, in("a0") name.as_ptr(), in("a1") name.len(), lateout("a0") ret); } ret }
fn sys_file_read(name: &str, buf: &mut [u8]) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_READ, in("a0") name.as_ptr(), in("a1") name.len(), in("a2") buf.as_mut_ptr(), in("a3") buf.len(), lateout("a0") ret); } ret }
fn sys_file_list(index: usize, buf: &mut [u8]) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LIST, in("a0") index, in("a1") buf.as_mut_ptr(), in("a2") buf.len(), lateout("a0") ret); } ret }
fn sys_exec(data: &[u8]) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_EXEC, in("a0") data.as_ptr(), in("a1") data.len(), lateout("a0") ret); } ret }
#[allow(dead_code)]
fn sys_exit(code: i32) -> ! { unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_EXIT, in("a0") code); } loop {} }

struct UserOut;
impl fmt::Write for UserOut {
    fn write_str(&mut self, s: &str) -> fmt::Result { for c in s.bytes() { sys_putchar(c); } Ok(()) }
}
#[macro_export]
macro_rules! user_println { ($($arg:tt)*) => ({ use core::fmt::Write; let mut w = UserOut; let _ = write!(w, $($arg)*); let _ = write!(w, "\n"); }); }
#[macro_export]
macro_rules! user_print { ($($arg:tt)*) => ({ use core::fmt::Write; let mut w = UserOut; let _ = write!(w, $($arg)*); }); }

// --- Shell Task ---
fn task1() -> ! {
    user_println!("Shell initialized (Interrupt Driven).");
    let mut command = String::new();
    user_print!("eos> ");

    loop {
        // 從 Buffer 讀取
        let c = sys_getchar();
        if c != 0 {
            if c == 13 || c == 10 {
                user_println!("");
                let cmd_line = command.trim();
                let parts: Vec<&str> = cmd_line.split_whitespace().collect();
                if !parts.is_empty() {
                    match parts[0] {
                        "help" => user_println!("ls, cat <file>, exec <file>"),
                        "ls" => {
                            let mut idx = 0; let mut buf = [0u8; 32];
                            loop {
                                let len = sys_file_list(idx, &mut buf);
                                if len < 0 { break; }
                                let name = core::str::from_utf8(&buf[0..len as usize]).unwrap();
                                user_println!(" - {}", name); idx += 1;
                            }
                        },
                        "cat" => {
                            if parts.len() < 2 { user_println!("Usage: cat <file>"); }
                            else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 { user_println!("File not found."); }
                                else {
                                    let mut content = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut content);
                                    if let Ok(s) = core::str::from_utf8(&content) { user_println!("{}", s); }
                                    else { user_println!("(Binary)"); }
                                }
                            }
                        },
                        "exec" => {
                            if parts.len() < 2 { user_println!("Usage: exec <file>"); }
                            else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 { user_println!("File not found."); }
                                else {
                                    let mut elf_data = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut elf_data);
                                    user_println!("Loading {}...", fname);
                                    sys_exec(&elf_data);
                                }
                            }
                        },
                        _ => user_println!("Unknown: {}", parts[0]),
                    }
                }
                command.clear(); user_print!("eos> ");
            } else if c == 127 || c == 8 {
                if !command.is_empty() { command.pop(); sys_putchar(8); sys_putchar(b' '); sys_putchar(8); }
            } else { sys_putchar(c); command.push(c as char); }
        }
        for _ in 0..1000 {}
    }
}

fn task2() -> ! {
    loop { for _ in 0..5000000 {} }
}

// --- 中斷處理器集合 ---

// [修正 3] 將 ctx_ptr 改名為 _ctx_ptr 以消除未使用警告
#[unsafe(no_mangle)]
pub extern "C" fn handle_timer(_ctx_ptr: *mut Context) -> *mut Context {
    set_next_timer();
    unsafe {
        let next_task = if CURR_TASK == 1 { CURR_TASK = 2; &raw mut TASK2 } else { CURR_TASK = 1; &raw mut TASK1 };
        let next_ctx = if CURR_TASK == 1 { &raw mut CTX1 } else { &raw mut CTX2 };

        let next_root_ppn = (*next_task).root_ppn;
        let satp_val = if next_root_ppn != 0 {
            (8 << 60) | next_root_ppn
        } else {
            let kernel_root = crate::mm::page_table::KERNEL_PAGE_TABLE as usize;
            (8 << 60) | (kernel_root >> 12)
        };
        core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
        return next_ctx;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_external(ctx_ptr: *mut Context) -> *mut Context {
    unsafe {
        let irq = PLIC_CLAIM.read_volatile();
        if irq == 10 { // UART IRQ
            while let Some(c) = uart::_getchar() {
                push_key(c);
            }
        }
        PLIC_CLAIM.write_volatile(irq);
    }
    ctx_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }
    
    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
        println!("[Kernel] Unexpected interrupt: {}", code);
        return ctx_ptr;
    } else {
        if code == 8 { // Syscall
            unsafe {
                let id = (*ctx_ptr).regs[17];
                let a0 = (*ctx_ptr).regs[10];
                let a1 = (*ctx_ptr).regs[11];
                let a2 = (*ctx_ptr).regs[12];
                let a3 = (*ctx_ptr).regs[13];

                match id {
                    SYSCALL_PUTCHAR => print!("{}", a0 as u8 as char),
                    SYSCALL_GETCHAR => (*ctx_ptr).regs[10] = pop_key().unwrap_or(0) as u64,
                    SYSCALL_FILE_LEN => {
                        let slice = core::slice::from_raw_parts(a0 as *const u8, a1 as usize);
                        let fname = core::str::from_utf8(slice).unwrap_or("");
                        if let Some(data) = fs::get_file_content(fname) { (*ctx_ptr).regs[10] = data.len() as u64; }
                        else { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                    },
                    SYSCALL_FILE_READ => {
                        let slice = core::slice::from_raw_parts(a0 as *const u8, a1 as usize);
                        let fname = core::str::from_utf8(slice).unwrap_or("");
                        let user_buf = core::slice::from_raw_parts_mut(a2 as *mut u8, a3 as usize);
                        if let Some(data) = fs::get_file_content(fname) {
                            let len = core::cmp::min(data.len(), user_buf.len());
                            user_buf[..len].copy_from_slice(&data[..len]);
                            (*ctx_ptr).regs[10] = len as u64;
                        } else { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                    },
                    SYSCALL_FILE_LIST => {
                        let user_buf = core::slice::from_raw_parts_mut(a1 as *mut u8, a2 as usize);
                        let files = fs::list_files();
                        if (a0 as usize) < files.len() {
                            let fname = files[a0 as usize].as_bytes();
                            let len = core::cmp::min(fname.len(), user_buf.len());
                            user_buf[..len].copy_from_slice(&fname[..len]);
                            (*ctx_ptr).regs[10] = len as u64;
                        } else { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                    },
                    SYSCALL_EXEC => {
                        let elf_data = core::slice::from_raw_parts(a0 as *const u8, a1 as usize);
                        println!("[Kernel] Executing new process...");
                        let new_table = mm::page_table::new_user_page_table();
                        if new_table.is_null() { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                        else {
                            if let Some(entry) = elf::load_elf(elf_data, &mut *new_table) {
                                println!("[Kernel] ELF loaded. Switching SATP.");
                                if CURR_TASK == 1 { (*(&raw mut TASK1)).root_ppn = (new_table as usize) >> 12; }
                                else { (*(&raw mut TASK2)).root_ppn = (new_table as usize) >> 12; }
                                let satp_val = (8 << 60) | ((new_table as usize) >> 12);
                                core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
                                (*ctx_ptr).mepc = entry;
                                return ctx_ptr;
                            } else { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                        }
                    },
                    SYSCALL_EXIT => {
                        println!("[Kernel] User App exited with code: {}", a0);
                        let kernel_root = crate::mm::page_table::KERNEL_PAGE_TABLE as usize;
                        let satp_val = (8 << 60) | (kernel_root >> 12);
                        core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
                        if CURR_TASK == 1 { (*(&raw mut TASK1)).root_ppn = 0; }
                        else { (*(&raw mut TASK2)).root_ppn = 0; }
                        println!("Rebooting shell...");
                        (*ctx_ptr).mepc = task1 as u64;
                        return ctx_ptr;
                    },
                    _ => println!("Unknown Syscall: {}", id),
                }
                (*ctx_ptr).mepc += 4;
                return ctx_ptr;
            }
        }
        
        let mtval: usize;
        unsafe { core::arch::asm!("csrr {}, mtval", out(reg) mtval); }
        println!("\n[Crash] mcause={}, mepc={:x}, mtval={:x}", code, unsafe { (*ctx_ptr).mepc }, mtval);
        println!("Rebooting shell...");
        unsafe {
            let kernel_root = crate::mm::page_table::KERNEL_PAGE_TABLE as usize;
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) (8 << 60) | (kernel_root >> 12));
            if CURR_TASK == 1 { (*(&raw mut TASK1)).root_ppn = 0; }
            else { (*(&raw mut TASK2)).root_ppn = 0; }
            (*ctx_ptr).mepc = task1 as u64;
            let mut mstatus: usize;
            core::arch::asm!("csrr {}, mstatus", out(reg) mstatus);
            mstatus &= !(3 << 11); mstatus |= 1 << 7;
            core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
        }
        return ctx_ptr;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS with PLIC Interrupts        ");
    println!("-----------------------------------");

    unsafe {
        core::arch::asm!("csrw pmpaddr0, {}", in(reg) !0usize);
        core::arch::asm!("csrw pmpcfg0, {}", in(reg) 0x1Fusize);

        mm::frame::init();
        println!("[Kernel] Frame Allocator initialized.");

        let root_ptr = mm::frame::alloc_frame() as *mut PageTable;
        let root = &mut *root_ptr;
        mm::page_table::KERNEL_PAGE_TABLE = root_ptr;

        mm::page_table::map(root, 0x1000_0000, 0x1000_0000, PTE_R | PTE_W);
        let mut addr = 0x0200_0000;
        while addr < 0x0200_FFFF { mm::page_table::map(root, addr, addr, PTE_R | PTE_W); addr += 4096; }
        
        println!("[Kernel] Mapping PLIC...");
        let mut addr = 0x0C00_0000;
        let end_plic = 0x0C20_1000; 
        while addr < end_plic {
            mm::page_table::map(root, addr, addr, PTE_R | PTE_W);
            addr += 4096;
        }
        
        let start = 0x8000_0000; let end = 0x8040_0000; 
        let mut addr = start;
        while addr < end { mm::page_table::map(root, addr, addr, PTE_R | PTE_W | PTE_X | PTE_U); addr += 4096; }

        let satp_val = (8 << 60) | ((root_ptr as usize) >> 12);
        core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
        println!("[Kernel] MMU Enabled.");

        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        CTX1.regs[2] = stack1_top as u64; CTX1.mepc = task1 as u64;
        (*(&raw mut TASK1)).root_ppn = 0; 
        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        CTX2.regs[2] = stack2_top as u64; CTX2.mepc = task2 as u64;
        (*(&raw mut TASK2)).root_ppn = 0;

        init_plic();
        println!("[Kernel] PLIC Initialized.");

        core::arch::asm!("csrw mtvec, {}", in(reg) (trap_vector as usize) | 1);
        core::arch::asm!("csrw mscratch, {}", in(reg) &raw mut CTX1);
        
        let mstatus: usize = (0 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
        
        set_next_timer();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) (1 << 11) | (1 << 7));

        println!("[OS] Jumping to User Mode...");
        core::arch::asm!("mv sp, {}", "csrw mepc, {}", "mret", in(reg) CTX1.regs[2], in(reg) CTX1.mepc);
    }
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! { println!("\n[PANIC] {}", info); loop {} }