// === FILE: ./user_app/src/bin/pid.rs ===
#![no_std]
#![no_main]
#[macro_use] extern crate ulib;

fn main(_args: &[*const u8]) -> i32 {
    let pid = ulib::sys_getpid();
    println!("Hello! My PID is: {}", pid);
    
    for i in 1..=3 {
        println!("PID {} is working... round {}", pid, i);
        // 主動讓出 CPU
        ulib::sys_yield();
    }
    
    println!("PID {} finished.", pid);
    0
}
entry_point!(main);