#![no_std]
#![no_main]

use core::panic::PanicInfo;

const SYSCALL_PUTCHAR: u64 = 1;
const SYSCALL_EXIT: u64 = 93;

fn sys_putchar(c: u8) {
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_PUTCHAR, in("a0") c); }
}

fn sys_exit(code: i32) -> ! {
    unsafe { core::arch::asm!("ecall", in("a7") SYSCALL_EXIT, in("a0") code); }
    loop {}
}

struct Console;
impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() { sys_putchar(c); }
        Ok(())
    }
}

// [修正] 增加 argc 和 argv 參數
// 根據 RISC-V 呼叫慣例：
// a0 = argc (usize)
// a1 = argv (*const *const u8) -> 指向字串指標陣列的指標
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start(argc: usize, argv: *const *const u8) -> ! {
    use core::fmt::Write;
    let mut out = Console;

    let _ = write!(out, "\n[UserApp] Started!\n");
    let _ = write!(out, "[UserApp] argc = {}\n", argc);

    // 遍歷並印出所有參數
    for i in 0..argc {
        unsafe {
            // 1. 取得第 i 個字串的指標 (argv[i])
            let str_ptr = *argv.add(i);
            
            // 2. 計算字串長度 (尋找 \0)
            let mut len = 0;
            while *str_ptr.add(len) != 0 {
                len += 1;
            }

            // 3. 轉成 Rust slice 並印出
            let slice = core::slice::from_raw_parts(str_ptr, len);
            let s = core::str::from_utf8(slice).unwrap_or("<?>");
            let _ = write!(out, "[UserApp] argv[{}] = \"{}\"\n", i, s);
        }
    }

    sys_exit(0);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }