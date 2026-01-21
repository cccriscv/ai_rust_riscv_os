#![no_std]
#![no_main]
#[macro_use] extern crate ulib;

fn main(_args: &[*const u8]) -> i32 {
    let mut idx = 0;
    let mut buf = [0u8; 32];
    loop {
        let len = ulib::sys_file_list(idx, &mut buf);
        if len < 0 { break; }
        let name = core::str::from_utf8(&buf[0..len as usize]).unwrap_or("???");
        println!(" - {}", name);
        idx += 1;
    }
    0
}
entry_point!(main);