#![no_std]
#![no_main]

#[macro_use]
extern crate ulib;

fn main(_args: &[*const u8]) -> i32 {
    println!("Hello from the Unified Project!");
    0
}

entry_point!(main);