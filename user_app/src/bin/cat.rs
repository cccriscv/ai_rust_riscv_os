#![no_std]
#![no_main]
#[macro_use] extern crate ulib;

fn main(args: &[*const u8]) -> i32 {
    if args.len() < 2 {
        println!("Usage: cat <file>");
        return 1;
    }
    // 解析檔名 (args[1])
    let filename = unsafe {
        let ptr = args[1];
        let mut len = 0;
        while *ptr.add(len) != 0 { len += 1; }
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8(slice).unwrap_or("")
    };

    let f_len = ulib::sys_file_len(filename);
    if f_len < 0 {
        println!("File not found.");
        return 1;
    }

    let mut buf = [0u8; 512]; 
    let read_len = ulib::sys_file_read(filename, &mut buf);
    if read_len > 0 {
        if let Ok(s) = core::str::from_utf8(&buf[0..read_len as usize]) {
            println!("{}", s);
        } else {
            println!("(Binary file)");
        }
    }
    0
}
entry_point!(main);