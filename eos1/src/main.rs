#![no_std]
#![no_main]

mod task;
use core::panic::PanicInfo;
use task::{Task, Context, STACK_SIZE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    fn switch_to(old: *mut Context, new: *const Context);
}

fn uart_putc(c: u8) {
    let uart = 0x1000_0000 as *mut u8;
    unsafe { *uart = c };
}

fn uart_puts(s: &str) {
    for c in s.bytes() {
        uart_putc(c);
    }
}

// 將 Task 宣告為全域靜態變數，這樣位址就不會動了
static mut TASK1: Task = Task::new();
static mut TASK2: Task = Task::new();
static mut TASK1_CTX: Context = Context::empty();
static mut TASK2_CTX: Context = Context::empty();
static mut MAIN_CTX: Context = Context::empty();

fn task1() -> ! {
    loop {
        uart_puts("Task 1 running...\n");
        for _ in 0..1000000 {} 
        unsafe { switch_to(&raw mut TASK1_CTX, &raw const TASK2_CTX) };
    }
}

fn task2() -> ! {
    loop {
        uart_puts("Task 2 running...\n");
        for _ in 0..1000000 {}
        unsafe { switch_to(&raw mut TASK2_CTX, &raw const TASK1_CTX) };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    uart_puts("Starting Multi-tasking OS...\n");

    unsafe {
        // 初始化 Task 1 的上下文
        let stack1_top = (&raw mut TASK1.stack as usize) + STACK_SIZE;
        TASK1_CTX.sp = stack1_top as u64;
        TASK1_CTX.ra = task1 as u64;

        // 初始化 Task 2 的上下文
        let stack2_top = (&raw mut TASK2.stack as usize) + STACK_SIZE;
        TASK2_CTX.sp = stack2_top as u64;
        TASK2_CTX.ra = task2 as u64;

        uart_puts("Context initialized, switching...\n");
        
        // 第一次切換
        switch_to(&raw mut MAIN_CTX, &raw const TASK1_CTX);
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}