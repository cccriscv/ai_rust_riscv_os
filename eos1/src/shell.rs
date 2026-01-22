// === FILE: ./eos1/src/shell.rs ===
use crate::syscall::*; 
use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;

// --- Syscall Wrappers ---

fn sys_putchar(c: u8) { 
    unsafe { core::arch::asm!("ecall", in("a7") PUTCHAR, in("a0") c); } 
}

fn sys_getchar() -> u8 { 
    let mut ret: usize; 
    unsafe { core::arch::asm!("ecall", in("a7") GETCHAR, lateout("a0") ret); } 
    ret as u8 
}

fn sys_file_len(name: &str) -> isize { 
    let mut ret: isize; 
    unsafe { core::arch::asm!("ecall", in("a7") FILE_LEN, in("a0") name.as_ptr(), in("a1") name.len(), lateout("a0") ret); } 
    ret 
}

fn sys_file_read(name: &str, buf: &mut [u8]) -> isize { 
    let mut ret: isize; 
    unsafe { core::arch::asm!("ecall", in("a7") FILE_READ, in("a0") name.as_ptr(), in("a1") name.len(), in("a2") buf.as_mut_ptr(), in("a3") buf.len(), lateout("a0") ret); } 
    ret 
}

fn sys_file_list(index: usize, buf: &mut [u8]) -> isize { 
    let mut ret: isize; 
    unsafe { core::arch::asm!("ecall", in("a7") FILE_LIST, in("a0") index, in("a1") buf.as_mut_ptr(), in("a2") buf.len(), lateout("a0") ret); } 
    ret 
}

fn sys_exec(data: &[u8], argv: &[&str]) -> isize { 
    let mut ret: isize; 
    unsafe { 
        core::arch::asm!(
            "ecall", 
            in("a7") EXEC, 
            in("a0") data.as_ptr(), 
            in("a1") data.len(), 
            in("a2") argv.as_ptr(), 
            in("a3") argv.len(), 
            lateout("a0") ret
        ); 
    } 
    ret 
}

fn sys_disk_read(sector: u64, buf: &mut [u8]) { 
    unsafe { 
        core::arch::asm!(
            "ecall", 
            in("a7") DISK_READ, 
            in("a0") sector, 
            in("a1") buf.as_mut_ptr(), 
            in("a2") buf.len()
        ); 
    } 
}

fn sys_file_write(name: &str, data: &[u8]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") FILE_WRITE,
            in("a0") name.as_ptr(),
            in("a1") name.len(),
            in("a2") data.as_ptr(),
            in("a3") data.len(),
            lateout("a0") ret,
        );
    }
    ret
}

fn sys_chdir(name: &str) -> isize { 
    let mut ret: isize; 
    unsafe { core::arch::asm!("ecall", in("a7") CHDIR, in("a0") name.as_ptr(), in("a1") name.len(), lateout("a0") ret); } 
    ret 
}

// [新增] Yield: 主動讓出 CPU
fn sys_yield() { 
    unsafe { core::arch::asm!("ecall", in("a7") SCHED_YIELD); } 
}

// [新增] Wait: 等待子行程結束
// 回傳值: >0 (子行程 PID), -1 (子行程仍在執行), -2 (無子行程)
fn sys_wait(status: &mut i32) -> isize {
    let mut ret: isize;
    unsafe { 
        core::arch::asm!(
            "ecall", 
            in("a7") WAIT, 
            in("a0") -1, // 等待任意子行程
            in("a1") status as *mut i32, 
            lateout("a0") ret
        ); 
    }
    ret
}

// --- Output Helpers ---

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
        let mut w = crate::shell::UserOut; 
        let _ = write!(w, $($arg)*); 
        let _ = write!(w, "\n"); 
    }); 
}

#[macro_export]
macro_rules! user_print { 
    ($($arg:tt)*) => ({ 
        use core::fmt::Write; 
        let mut w = crate::shell::UserOut; 
        let _ = write!(w, $($arg)*); 
    }); 
}

// --- Helpers ---

fn parse_int(s: &str) -> Option<u64> {
    let mut res: u64 = 0;
    for c in s.bytes() {
        if c >= b'0' && c <= b'9' { 
            res = res * 10 + (c - b'0') as u64; 
        } else { 
            return None; 
        }
    }
    Some(res)
}

// 支援引號的參數解析器
fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for c in input.chars() {
        match c {
            '"' => { in_quote = !in_quote; } // 切換引號狀態
            c if c.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => { current.push(c); }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

// --- Tasks ---

pub extern "C" fn shell_entry() -> ! {
    user_println!("Shell initialized (Sync Mode).");
    let mut command = String::new();
    user_print!("eos> ");

    loop {
        let c = sys_getchar();
        if c != 0 {
            if c == 13 || c == 10 { // Enter
                user_println!("");
                let cmd_line = command.trim();
                
                let parts = parse_args(cmd_line);
                
                if !parts.is_empty() {
                    match parts[0].as_str() {
                        "help" => user_println!("ls, cat <file>, write <file> \"text\", exec <file> [args], cd <dir>, dread <sector>, memtest, panic"),
                        
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
                            if parts.len() < 2 { user_println!("Usage: cat <file>"); }
                            else {
                                let fname = &parts[1];
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

                        "cd" => {
                            if parts.len() < 2 { user_println!("Usage: cd <dir>"); }
                            else {
                                let ret = sys_chdir(&parts[1]);
                                if ret == 0 { user_println!("Changed directory."); }
                                else { user_println!("Directory not found."); }
                            }
                        },

                        "write" => {
                            if parts.len() < 3 {
                                user_println!("Usage: write <filename> \"content\"");
                            } else {
                                let fname = &parts[1];
                                let content = &parts[2]; 
                                user_println!("Writing to {}...", fname);
                                let ret = sys_file_write(fname, content.as_bytes());
                                if ret == 0 { user_println!("Success!"); } else { user_println!("Failed (Error: {})", ret); }
                            }
                        },
                        
                        "exec" => {
                            if parts.len() < 2 { user_println!("Usage: exec <file> [args...]"); }
                            else {
                                let fname = &parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 { user_println!("File not found."); }
                                else {
                                    let mut elf_data = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut elf_data);
                                    
                                    let args_vec: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
                                    
                                    // 1. 建立並執行子行程
                                    let pid = sys_exec(&elf_data, &args_vec);
                                    
                                    if pid > 0 {
                                        // 2. 同步等待子行程結束
                                        let mut status = 0;
                                        loop {
                                            let wpid = sys_wait(&mut status);
                                            if wpid > 0 {
                                                // 子行程已結束並被回收
                                                break;
                                            } else if wpid == -1 {
                                                // 子行程仍在執行，Shell 主動讓出 CPU
                                                sys_yield();
                                            } else {
                                                // 無子行程 (理論上不應發生)
                                                break;
                                            }
                                        }
                                    } else {
                                        user_println!("Exec failed.");
                                    }
                                }
                            }
                        },
                        
                        "dread" => {
                            let sector_str = parts.get(1).map(|s| s.as_str()).unwrap_or("0");
                            let sector = parse_int(sector_str).unwrap_or(0);
                            let mut buf = [0u8; 512];
                            user_println!("Reading sector {}...", sector);
                            sys_disk_read(sector, &mut buf);
                            if let Ok(s) = core::str::from_utf8(&buf[0..64]) { user_println!("Data: {}", s); }
                            else { user_println!("Data: {:x?}", &buf[0..16]); }
                        },
                        
                        "memtest" => {
                            for i in 0..1000 { let mut v = Vec::new(); v.push(i); }
                            user_println!("Memtest done.");
                        },
                        
                        "panic" => unsafe { (0x0 as *mut u8).write_volatile(0); },
                        
                        _ => user_println!("Unknown: {}", parts[0]),
                    }
                }
                command.clear(); 
                user_print!("eos> ");
            } 
            else if c == 127 || c == 8 { // Backspace
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

pub extern "C" fn bg_task() -> ! {
    loop { for _ in 0..5000000 {} }
}