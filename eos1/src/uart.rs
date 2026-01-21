use core::fmt;

pub struct Uart {
    base_address: *mut u8,
}

impl Uart {
    pub const fn new(addr: usize) -> Self {
        Self {
            base_address: addr as *mut u8,
        }
    }

    pub fn putc(&self, c: u8) {
        unsafe {
            // 寫入 UART 暫存器
            self.base_address.write_volatile(c);
        }
    }
}

// 實作 Write Trait，這樣才能使用 Rust 的格式化功能
impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.putc(byte);
        }
        Ok(())
    }
}

// 建立一個全域的 UART 實例
pub static mut WRITER: Uart = Uart::new(0x1000_0000);

// src/uart.rs

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        // 1. 取得 WRITER 的原生可變指標 (raw mutable pointer)
        let writer_ptr = &raw mut WRITER;
        
        // 2. 透過原生指標呼叫 write_fmt
        // 在 Rust 中，(*ptr).method() 可以用來在原生指標上呼叫方法
        (*writer_ptr).write_fmt(args).unwrap();
    }
}

// src/uart.rs (續)

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}