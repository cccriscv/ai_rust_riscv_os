// === FILE: ./user_app/src/lib.rs ===
#![no_std]

use core::fmt;

// --- System Call ID ---
pub const SYSCALL_PUTCHAR: u64 = 1;
pub const SYSCALL_GETCHAR: u64 = 2;
pub const SYSCALL_FILE_LEN: u64 = 3;
pub const SYSCALL_FILE_READ: u64 = 4;
pub const SYSCALL_FILE_LIST: u64 = 5;
pub const SYSCALL_EXEC: u64 = 6;
pub const SYSCALL_DISK_READ: u64 = 7;
pub const SYSCALL_FILE_WRITE: u64 = 8;
pub const SYSCALL_EXIT: u64 = 93;

// --- Wrappers ---

pub fn sys_putchar(c: u8) {
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c); }
}

pub fn sys_exit(code: i32) -> ! {
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_EXIT, in("a0") code); }
    loop {}
}

pub fn sys_file_len(name: &str) -> isize {
    let mut ret: isize;
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LEN, in("a0") name.as_ptr(), in("a1") name.len(), lateout("a0") ret); }
    ret
}

pub fn sys_file_read(name: &str, buf: &mut [u8]) -> isize {
    let mut ret: isize;
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_READ, in("a0") name.as_ptr(), in("a1") name.len(), in("a2") buf.as_mut_ptr(), in("a3") buf.len(), lateout("a0") ret); }
    ret
}

pub fn sys_file_list(index: usize, buf: &mut [u8]) -> isize {
    let mut ret: isize;
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_FILE_LIST, in("a0") index, in("a1") buf.as_mut_ptr(), in("a2") buf.len(), lateout("a0") ret); }
    ret
}

// --- Println ---

pub struct Console;
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() { sys_putchar(c); }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    let mut out = Console;
    out.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// --- Entry Point Macro ---

#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text.entry")]
        pub extern "C" fn _start(argc: usize, argv: *const *const u8) -> ! {
            let args: &[*const u8] = unsafe { 
                if argv.is_null() { &[] } 
                else { core::slice::from_raw_parts(argv, argc) }
            };
            
            let code = $path(args);
            
            $crate::sys_exit(code);
        }

        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            $crate::println!("\n[User Panic]");
            $crate::println!("{}", info);
            $crate::sys_exit(-1);
        }
    };
}