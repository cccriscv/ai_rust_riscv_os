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

// 堆疊大小 16KB
pub const STACK_SIZE: usize = 16384;

#[repr(C, align(16))]
pub struct Task {
    pub stack: [u8; STACK_SIZE],
    // [新增] 根分頁表的實體頁號 (Physical Page Number)
    // 如果為 0，代表使用核心預設分頁表
    pub root_ppn: usize, 
}

impl Task {
    pub const fn new() -> Self {
        Self { 
            stack: [0; STACK_SIZE],
            root_ppn: 0, 
        }
    }
}