#![no_std]
#![no_main]

#[macro_use]
mod uart;
mod task;
mod heap;
mod fs;
mod elf;

#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

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

// --- Syscall ID ---
const SYSCALL_PUTCHAR: u64 = 1;
const SYSCALL_GETCHAR: u64 = 2;
const SYSCALL_FILE_LEN: u64 = 3;
const SYSCALL_FILE_READ: u64 = 4;
const SYSCALL_FILE_LIST: u64 = 5;
const SYSCALL_EXEC: u64 = 6; // [新增] Exec System Call

// --- User Syscall Wrappers ---
fn sys_putchar(c: u8) { unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c); } }
fn sys_getchar() -> u8 { let mut ret: usize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_GETCHAR, lateout("a0") ret); } ret as u8 }
fn sys_file_len(name: &str) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LEN, in("a0") name.as_ptr(), in("a1") name.len(), lateout("a0") ret); } ret }
fn sys_file_read(name: &str, buf: &mut [u8]) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_READ, in("a0") name.as_ptr(), in("a1") name.len(), in("a2") buf.as_mut_ptr(), in("a3") buf.len(), lateout("a0") ret); } ret }
fn sys_file_list(index: usize, buf: &mut [u8]) -> isize { let mut ret: isize; unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LIST, in("a0") index, in("a1") buf.as_mut_ptr(), in("a2") buf.len(), lateout("a0") ret); } ret }

/// [新增] 請求核心執行 ELF 檔
/// data: 包含 ELF 內容的 Slice
fn sys_exec(data: &[u8]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_EXEC,
            in("a0") data.as_ptr(),
            in("a1") data.len(),
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
    user_println!("Shell initialized. Type 'exec program.elf' to run.");
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
                        "help" => user_println!("ls, cat <file>, exec <file>"),
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
                            if parts.len() < 2 { user_println!("Usage: cat <filename>"); } 
                            else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 { user_println!("File not found."); } else {
                                    let mut content = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut content);
                                    if let Ok(s) = core::str::from_utf8(&content) { user_println!("{}", s); } 
                                    else { user_println!("(Binary)"); }
                                }
                            }
                        },
                        "exec" => {
                            if parts.len() < 2 {
                                user_println!("Usage: exec <filename>");
                            } else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 {
                                    user_println!("File not found.");
                                } else {
                                    // 1. Shell 讀取檔案到 User Memory (Heap)
                                    let mut elf_data = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut elf_data);
                                    
                                    user_println!("Loading {}...", fname);
                                    
                                    // 2. 呼叫 System Call，讓核心去處理載入與跳轉
                                    let ret = sys_exec(&elf_data);
                                    
                                    if ret < 0 {
                                        user_println!("Exec failed! Invalid ELF.");
                                    }
                                    // 如果成功，sys_exec 不會返回，因為核心已經跳轉到新程式了
                                }
                            }
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

fn task2() -> ! {
    loop { for _ in 0..5000000 {} }
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
                    SYSCALL_GETCHAR => { (*ctx_ptr).regs[10] = uart::_getchar().unwrap_or(0) as u64; },
                    SYSCALL_FILE_LEN => {
                        let ptr = a0 as *const u8;
                        let len = a1 as usize;
                        let slice = core::slice::from_raw_parts(ptr, len);
                        let fname = core::str::from_utf8(slice).unwrap_or("");
                        if let Some(data) = fs::get_file_content(fname) { (*ctx_ptr).regs[10] = data.len() as u64; } 
                        else { (*ctx_ptr).regs[10] = (-1isize) as u64; }
                    },
                    SYSCALL_FILE_READ => {
                        let fname = core::str::from_utf8(core::slice::from_raw_parts(a0 as *const u8, a1 as usize)).unwrap_or("");
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
                    // [新增] EXEC 實作
                    SYSCALL_EXEC => {
                        let data_ptr = a0 as *const u8;
                        let data_len = a1 as usize;
                        let elf_data = core::slice::from_raw_parts(data_ptr, data_len);
                        
                        println!("[Kernel] Loading ELF...");
                        // 核心在 M-Mode，可以寫入任意記憶體，不會觸發 Access Fault
                        if let Some(entry) = elf::load_elf(elf_data) {
                            println!("[Kernel] Jumping to {:x}", entry);
                            // 修改當前任務的 mepc 為程式進入點
                            (*ctx_ptr).mepc = entry;
                            // [重要] 這裡不能 +4，因為我們要跳到新程式的第一行，而不是 ecall 的下一行
                            return ctx_ptr;
                        } else {
                            (*ctx_ptr).regs[10] = (-1isize) as u64; // Failed
                        }
                    }
                    _ => println!("Unknown Syscall: {}", id),
                }
                (*ctx_ptr).mepc += 4; // 其他 System Call 都要 +4
                return ctx_ptr;
            }
        } else {
            // [修正] 處理 User App 崩潰，並防止陷入 M-Mode 迴圈
            println!("\n[Trap caught] mcause={}, mepc={:x}", code, unsafe { (*ctx_ptr).mepc });
            println!("User App terminated. Rebooting shell...");
            
            unsafe { 
                // 1. 重設 PC 到 Shell 的開頭
                (*ctx_ptr).mepc = task1 as u64; 
                
                // 2. [關鍵修正] 強制設定 mstatus.MPP 為 User Mode (00)
                // 否則如果崩潰發生在 Kernel (M-Mode)，mret 會返回 M-Mode，導致 task1 權限過高
                let mut mstatus: usize;
                core::arch::asm!("csrr {}, mstatus", out(reg) mstatus);
                
                // 清除 MPP 位元 (第 11, 12 位)，設為 00 (User Mode)
                mstatus &= !(3 << 11);
                
                // 確保 MPIE (第 7 位) 為 1，這樣返回後中斷才會開啟
                mstatus |= (1 << 7);
                
                core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
            }
            return ctx_ptr;
        }
    }
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS with ELF Loader (M-Mode)    ");
    println!("-----------------------------------");

    unsafe {
        core::arch::asm!("csrw pmpaddr0, {}", in(reg) !0usize);
        core::arch::asm!("csrw pmpcfg0, {}", in(reg) 0x1Fusize);

        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        CTX1.regs[2] = stack1_top as u64; CTX1.mepc = task1 as u64;
        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        CTX2.regs[2] = stack2_top as u64; CTX2.mepc = task2 as u64;

        core::arch::asm!("csrw mtvec, {}", in(reg) trap_entry);
        core::arch::asm!("csrw mscratch, {}", in(reg) &raw mut CTX1);
        let mstatus: usize = (0 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);

        set_next_timer();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) 1 << 7);

        println!("[OS] User Mode initialized.");
        core::arch::asm!("mv sp, {}", "csrw mepc, {}", "mret", in(reg) CTX1.regs[2], in(reg) CTX1.mepc);
    }
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[PANIC] {}", info);
    loop {}
}