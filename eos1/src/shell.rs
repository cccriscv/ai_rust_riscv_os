use crate::syscall::*; // 引用 syscall.rs 定義的常數
use alloc::vec::Vec;
use alloc::string::String;
use core::fmt;

// --- Syscall Wrappers (User Library) ---

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

// [新增] 檔案寫入系統呼叫
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

// --- Tasks ---

pub extern "C" fn shell_entry() -> ! {
    user_println!("Shell initialized (RW FS).");
    let mut command = String::new();
    user_print!("eos> ");

    loop {
        let c = sys_getchar();
        if c != 0 {
            if c == 13 || c == 10 { // Enter
                user_println!("");
                let cmd_line = command.trim();
                let parts: Vec<&str> = cmd_line.split_whitespace().collect();
                
                if !parts.is_empty() {
                    match parts[0] {
                        "help" => user_println!("ls, cat <file>, write <file> <content>, exec <file> [args], dread <sector>, memtest, panic"),
                        
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
                        "cd" => {
                            if parts.len() < 2 { user_println!("Usage: cd <dir>"); }
                            else {
                                let ret = sys_chdir(parts[1]);
                                if ret == 0 { user_println!("Changed directory."); }
                                else { user_println!("Directory not found."); }
                            }
                        },
                        // [新增] 寫入指令
                        "write" => {
                            if parts.len() < 3 {
                                user_println!("Usage: write <filename> <content>");
                            } else {
                                let fname = parts[1];
                                let content = parts[2]; // 簡單取第三個部分，不支援空白
                                
                                user_println!("Writing to {}...", fname);
                                let ret = sys_file_write(fname, content.as_bytes());
                                
                                if ret == 0 {
                                    user_println!("Success!");
                                } else {
                                    user_println!("Failed (Error: {})", ret);
                                }
                            }
                        },
                        
                        "exec" => {
                            if parts.len() < 2 { user_println!("Usage: exec <file> [args...]"); }
                            else {
                                let fname = parts[1];
                                let len = sys_file_len(fname);
                                if len < 0 { user_println!("File not found."); }
                                else {
                                    let mut elf_data = vec![0u8; len as usize];
                                    sys_file_read(fname, &mut elf_data);
                                    
                                    let args = &parts[1..];
                                    user_println!("Loading {} with args {:?}...", fname, args);
                                    sys_exec(&elf_data, args);
                                }
                            }
                        },
                        
                        "dread" => {
                            if parts.len() < 2 { user_println!("Usage: dread <sector>"); }
                            else {
                                let sector = parse_int(parts[1]).unwrap_or(0);
                                let mut buf = [0u8; 512];
                                user_println!("Reading sector {}...", sector);
                                sys_disk_read(sector, &mut buf);
                                if let Ok(s) = core::str::from_utf8(&buf[0..64]) { user_println!("Data: {}", s); }
                                else { user_println!("Data: {:x?}", &buf[0..16]); }
                            }
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
        // Polling delay
        for _ in 0..1000 {}
    }
}

pub extern "C" fn bg_task() -> ! {
    loop { for _ in 0..5000000 {} }
}