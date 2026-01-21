#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;
mod heap;
mod fs;

// [修正 1] 加上 #[macro_use] 才能使用 vec! 巨集
#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
// [修正 2] 移除未使用的 format 引用
// use alloc::format;

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
const SYSCALL_GETCHAR: u64 = 2;
const SYSCALL_FILE_LEN: u64 = 3;
const SYSCALL_FILE_READ: u64 = 4;
const SYSCALL_FILE_LIST: u64 = 5;

fn sys_putchar(c: u8) {
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c); }
}

fn sys_getchar() -> u8 {
    let mut ret: usize;
    unsafe {
        core::arch::asm!("ecall", in("a7") SYSCALL_GETCHAR, lateout("a0") ret);
    }
    ret as u8
}

fn sys_file_len(name: &str) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_FILE_LEN,
            in("a0") name.as_ptr(),
            in("a1") name.len(),
            lateout("a0") ret,
        );
    }
    ret
}

fn sys_file_read(name: &str, buf: &mut [u8]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_FILE_READ,
            in("a0") name.as_ptr(),
            in("a1") name.len(),
            in("a2") buf.as_mut_ptr(),
            in("a3") buf.len(),
            lateout("a0") ret,
        );
    }
    ret
}

fn sys_file_list(index: usize, buf: &mut [u8]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_FILE_LIST,
            in("a0") index,
            in("a1") buf.as_mut_ptr(),
            in("a2") buf.len(),
            lateout("a0") ret,
        );
    }
    ret
}

struct UserOut;
impl fmt::Write for UserOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() { sys_putchar(c); }
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

// --- Shell Task ---

fn task1() -> ! {
    user_println!("Shell initialized. Type 'help' for commands.");
    let mut command = String::new();
    user_print!("eos> ");

    loop {
        let c = sys_getchar();
        if c != 0 {
            if c == 13 || c == 10 {
                user_println!("");
                let cmd_line = command.trim();
                let parts: Vec<&str> = cmd_line.split_whitespace().collect();
                
                if !parts.is_empty() {
                    match parts[0] {
                        "help" => {
                            user_println!("Commands: help, ls, cat <file>, panic");
                        },
                        "ls" => {
                            let mut idx = 0;
                            let mut buf = [0u8; 32];
                            loop {
                                let len = sys_file_list(idx, &mut buf);
                                if len < 0 { break; }
                                let name = core::str::from_utf8(&buf[0..len as usize]).unwrap();
                                user_println!(" - {}", name);
                                idx += 1;
                            }
                        },
                        "cat" => {
                            if parts.len() < 2 {
                                user_println!("Usage: cat <filename>");
                            } else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 {
                                    user_println!("File not found: {}", fname);
                                } else {
                                    // 這裡使用了 vec! 巨集，需要 #[macro_use]
                                    let mut content = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut content);
                                    
                                    if let Ok(s) = core::str::from_utf8(&content) {
                                        user_println!("--- begin {} ---", fname);
                                        user_println!("{}", s);
                                        user_println!("--- end ---");
                                    } else {
                                        user_println!("(Binary file)");
                                    }
                                }
                            }
                        },
                        "panic" => {
                             user_println!("Crashing...");
                             unsafe { (0x0 as *mut u8).write_volatile(0); }
                        },
                        _ => user_println!("Unknown: {}", parts[0]),
                    }
                }
                
                command.clear();
                user_print!("eos> ");
            } 
            else if c == 127 || c == 8 {
                if !command.is_empty() {
                    command.pop();
                    sys_putchar(8); sys_putchar(b' '); sys_putchar(8);
                }
            } else {
                sys_putchar(c);
                command.push(c as char);
            }
        }
        for _ in 0..1000 {} 
    }
}

// [修正 3] 移除未使用的變數 count，避免警告
fn task2() -> ! {
    loop {
        for _ in 0..5000000 {}
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
        if code == 7 { // Timer
            set_next_timer();
            unsafe {
                if CURR_TASK == 1 { CURR_TASK = 2; return &raw mut CTX2; }
                else { CURR_TASK = 1; return &raw mut CTX1; }
            }
        }
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
                    SYSCALL_GETCHAR => {
                         (*ctx_ptr).regs[10] = uart::_getchar().unwrap_or(0) as u64;
                    },
                    SYSCALL_FILE_LEN => {
                        let ptr = a0 as *const u8;
                        let len = a1 as usize;
                        let slice = core::slice::from_raw_parts(ptr, len);
                        let fname = core::str::from_utf8(slice).unwrap_or("");
                        
                        if let Some(data) = fs::get_file_content(fname) {
                            (*ctx_ptr).regs[10] = data.len() as u64;
                        } else {
                            (*ctx_ptr).regs[10] = (-1isize) as u64;
                        }
                    },
                    SYSCALL_FILE_READ => {
                        let name_ptr = a0 as *const u8;
                        let name_len = a1 as usize;
                        let name_slice = core::slice::from_raw_parts(name_ptr, name_len);
                        let fname = core::str::from_utf8(name_slice).unwrap_or("");

                        let buf_ptr = a2 as *mut u8;
                        let buf_len = a3 as usize;
                        let user_buf = core::slice::from_raw_parts_mut(buf_ptr, buf_len);

                        if let Some(data) = fs::get_file_content(fname) {
                            let copy_len = core::cmp::min(data.len(), user_buf.len());
                            user_buf[..copy_len].copy_from_slice(&data[..copy_len]);
                            (*ctx_ptr).regs[10] = copy_len as u64;
                        } else {
                            (*ctx_ptr).regs[10] = (-1isize) as u64;
                        }
                    },
                    SYSCALL_FILE_LIST => {
                        let idx = a0 as usize;
                        let buf_ptr = a1 as *mut u8;
                        let buf_len = a2 as usize;
                        let user_buf = core::slice::from_raw_parts_mut(buf_ptr, buf_len);
                        
                        let files = fs::list_files();
                        if idx < files.len() {
                            let fname = files[idx].as_bytes();
                            let copy_len = core::cmp::min(fname.len(), user_buf.len());
                            user_buf[..copy_len].copy_from_slice(&fname[..copy_len]);
                            (*ctx_ptr).regs[10] = copy_len as u64;
                        } else {
                            (*ctx_ptr).regs[10] = (-1isize) as u64;
                        }
                    }
                    _ => println!("Unknown Syscall: {}", id),
                }
                (*ctx_ptr).mepc += 4;
                return ctx_ptr;
            }
        } else {
            println!("\n[Segmentation Fault]");
            unsafe { (*ctx_ptr).mepc = task1 as u64; }
            return ctx_ptr;
        }
    }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS with RAM Filesystem         ");
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

        println!("[OS] Filesystem mounted. Jumping to User Mode...");

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