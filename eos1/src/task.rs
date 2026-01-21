#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Context {
    pub regs: [u64; 32], 
    pub mepc: u64,       
}

impl Context {
    pub const fn empty() -> Self {
        Self {
            regs: [0; 32],
            mepc: 0,
        }
    }
}

// 使用 16KB 堆疊以支援 println!
pub const STACK_SIZE: usize = 16384;

#[repr(C, align(16))]
pub struct Task {
    pub stack: [u8; STACK_SIZE],
}

impl Task {
    pub const fn new() -> Self {
        Self { stack: [0; STACK_SIZE] }
    }
}