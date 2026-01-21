#![no_std]
#![no_main]
#[macro_use] extern crate ulib;

fn main(args: &[*const u8]) -> i32 {
    println!("[User] Hello from program!");
    println!("[User] argc = {}", args.len());
    
    // 解析第一個參數 (程式名稱)
    if args.len() > 0 {
        let name = unsafe {
            let ptr = args[0];
            let mut len = 0;
            while *ptr.add(len) != 0 { len += 1; }
            let slice = core::slice::from_raw_parts(ptr, len);
            core::str::from_utf8(slice).unwrap_or("?")
        };
        println!("[User] My name is: {}", name);
    }
    0
}
entry_point!(main);