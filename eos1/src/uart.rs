use core::fmt;

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub const fn new(addr: usize) -> Self {
        Self {
            base_address: addr,
        }
    }

    /// 寫入一個字元
    pub fn putc(&self, c: u8) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    /// 讀取一個字元 (非阻塞)
    /// 如果有字元，回傳 Some(c)，否則回傳 None
    pub fn getc(&self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            // 讀取 LSR (Line Status Register, offset 5)
            // Bit 0 為 1 代表有資料可讀
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                // 讀取 RBR (Receiver Buffer Register, offset 0)
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.putc(byte);
        }
        Ok(())
    }
}

// 建立全域的 UART 實例
pub static mut WRITER: Uart = Uart::new(0x1000_0000);

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        let writer_ptr = &raw mut WRITER;
        (*writer_ptr).write_fmt(args).unwrap();
    }
}

// 供核心呼叫的讀取函式
pub fn _getchar() -> Option<u8> {
    unsafe {
        let writer_ptr = &raw mut WRITER;
        (*writer_ptr).getc()
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}