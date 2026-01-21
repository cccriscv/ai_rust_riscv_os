#![no_std]
#![no_main]

use core::panic::PanicInfo;

// 引入外部的組合語言啟動檔
core::arch::global_asm!(include_str!("entry.S"));

// 在最新的 Rust 中，no_mangle 必須包裹在 unsafe() 裡面
#[unsafe(no_mangle)] 
pub extern "C" fn rust_main() -> ! {
    // UART0 的位址 (QEMU virt 機器)
    let uart = 0x1000_0000 as *mut u8;

    for c in b"Hello, RISC-V OS!\n".iter() {
        unsafe {
            // 往 UART 位址寫入字元
            *uart = *c;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}