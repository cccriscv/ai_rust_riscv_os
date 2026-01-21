#![no_std]
#![no_main]

// 宣告模組，並加上 macro_use 讓巨集在全域可用
#[macro_use]
mod uart;
mod task;

use core::panic::PanicInfo;
use task::{Task, Context, STACK_SIZE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    fn switch_to(old: *mut Context, new: *const Context);
}

static mut TASK1: Task = Task::new();
static mut TASK2: Task = Task::new();
static mut TASK1_CTX: Context = Context::empty();
static mut TASK2_CTX: Context = Context::empty();
static mut MAIN_CTX: Context = Context::empty();

fn task1() -> ! {
    let mut count = 0;
    loop {
        // 以前：uart_puts("Task 1...\n");
        // 現在：可以使用強大的格式化功能！
        println!("Task 1: count = {}, addr = {:p}", count, &count);
        
        count += 1;
        for _ in 0..1000000 {} 
        unsafe { switch_to(&raw mut TASK1_CTX, &raw const TASK2_CTX) };
    }
}

fn task2() -> ! {
    let mut count = 0;
    loop {
        println!("Task 2: count = {}, addr = {:p}", count, &count);
        
        count += 1;
        for _ in 0..1000000 {}
        unsafe { switch_to(&raw mut TASK2_CTX, &raw const TASK1_CTX) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    // 測試 println!
    println!("-----------------------------------");
    println!("   Welcome to EOS (Eos Operating System)!");
    println!("   Arch: RISC-V 64-bit");
    println!("-----------------------------------");

    unsafe {
        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        TASK1_CTX.sp = stack1_top as u64;
        TASK1_CTX.ra = task1 as u64;

        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        TASK2_CTX.sp = stack2_top as u64;
        TASK2_CTX.ra = task2 as u64;

        println!("[OS] Context initialized, starting scheduler...");
        switch_to(&raw mut MAIN_CTX, &raw const TASK1_CTX);
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // 以前只能 loop，現在可以印出錯誤訊息了！
    println!("\n[KERNEL PANIC] {}", info);
    loop {}
}