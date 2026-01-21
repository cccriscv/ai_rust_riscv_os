use crate::uart;

pub const BASE: usize = 0x0c00_0000;
pub const PRIORITY: *mut u32 = BASE as *mut u32;
pub const ENABLE: *mut u32 = (BASE + 0x2000) as *mut u32;
pub const THRESHOLD: *mut u32 = (BASE + 0x200000) as *mut u32;
pub const CLAIM: *mut u32 = (BASE + 0x200004) as *mut u32;

// 鍵盤緩衝區 (原本在 main.rs)
const KEY_BUFFER_SIZE: usize = 256;
static mut KEY_BUFFER: [u8; KEY_BUFFER_SIZE] = [0; KEY_BUFFER_SIZE];
static mut KEY_HEAD: usize = 0;
static mut KEY_TAIL: usize = 0;

pub fn init() {
    unsafe {
        let writer = &raw mut uart::WRITER;
        (*writer).enable_interrupt();
        let irq_uart = 10; 
        PRIORITY.add(irq_uart).write_volatile(1);
        ENABLE.write_volatile(1 << irq_uart);
        THRESHOLD.write_volatile(0);
    }
}

pub fn push_key(c: u8) {
    unsafe {
        let next = (KEY_HEAD + 1) % KEY_BUFFER_SIZE;
        if next != KEY_TAIL { KEY_BUFFER[KEY_HEAD] = c; KEY_HEAD = next; }
    }
}

pub fn pop_key() -> Option<u8> {
    unsafe {
        if KEY_HEAD == KEY_TAIL { return None; }
        let c = KEY_BUFFER[KEY_TAIL];
        KEY_TAIL = (KEY_TAIL + 1) % KEY_BUFFER_SIZE;
        Some(c)
    }
}

// 處理 PLIC 中斷的邏輯
pub fn handle_interrupt() {
    unsafe {
        let irq = CLAIM.read_volatile();
        if irq == 10 { 
            while let Some(c) = uart::_getchar() { push_key(c); }
        }
        CLAIM.write_volatile(irq);
    }
}