#![no_std]
#![no_main]

use core::panic::PanicInfo;

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

const SYSCALL_EXIT: u64 = 93; // Linux 的 exit 號碼通常是 93

fn sys_exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SYSCALL_EXIT,
            in("a0") code,
        );
    }
    loop {} // 不會執行到這裡
}

struct Console;
impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() { sys_putchar(c); }
        Ok(())
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    let mut out = Console;

    let _ = write!(out, "\n[UserApp] Hello, World!\n");
    
    // [修正] 更新這行文字，或者我們用點小技巧印出函式指標位址
    let pc = _start as usize; 
    let _ = write!(out, "[UserApp] I am running at 0x{:x}\n", pc);
    
    let _ = write!(out, "[UserApp] Calculation: 10 + 20 = {}\n", 10 + 20);

    // [修正] 改用 sys_exit 優雅退出
    sys_exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}