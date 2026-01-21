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

struct Console;
impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() { sys_putchar(c); }
        Ok(())
    }
}

// [修正 1] 新版 Rust 要求 linker 相關屬性必須標記 unsafe
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    let mut out = Console;

    let _ = write!(out, "\n[UserApp] Hello, World!\n");
    let _ = write!(out, "[UserApp] I am running at 0x80200000\n");
    let _ = write!(out, "[UserApp] Calculation: 10 + 20 = {}\n", 10 + 20);

    // 觸發非法指令結束程式
    unsafe { core::arch::asm!("unimp"); }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}