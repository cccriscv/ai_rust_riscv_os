#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Context {
    pub ra: u64, pub sp: u64,
    pub s0: u64, pub s1: u64, pub s2: u64, pub s3: u64, pub s4: u64,
    pub s5: u64, pub s6: u64, pub s7: u64, pub s8: u64, pub s9: u64,
    pub s10: u64, pub s11: u64,
}

impl Context {
    pub const fn empty() -> Self {
        Self {
            ra: 0, sp: 0,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0,
            s5: 0, s6: 0, s7: 0, s8: 0, s9: 0,
            s10: 0, s11: 0,
        }
    }
}

pub const STACK_SIZE: usize = 4096;

#[repr(C, align(16))]
pub struct Task {
    pub stack: [u8; STACK_SIZE],
}

impl Task {
    pub const fn new() -> Self {
        Self { stack: [0; STACK_SIZE] }
    }
}