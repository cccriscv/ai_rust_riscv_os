// === FILE: ./eos1/src/task.rs ===
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::vec;
use crate::mm::page_table::KERNEL_PAGE_TABLE;

// 堆疊大小 16KB
pub const STACK_SIZE: usize = 16384;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct Context {
    pub regs: [u64; 32], 
    pub mepc: u64,       
}

impl Context {
    pub const fn empty() -> Self {
        Self { regs: [0; 32], mepc: 0 }
    }
}

#[repr(C, align(16))]
pub struct Task {
    #[allow(dead_code)]
    pub id: usize,
    #[allow(dead_code)]
    pub stack: Vec<u8>, 
    pub context: Context,
    pub root_ppn: usize,
    // [移除] files
}

impl Task {
    pub fn new_kernel(id: usize, entry: extern "C" fn() -> !) -> Self {
        let stack = vec![0u8; STACK_SIZE];
        let stack_top = stack.as_ptr() as usize + STACK_SIZE;
        let aligned_sp = stack_top & !0xF;

        let mut task = Self {
            id,
            stack,
            context: Context::empty(),
            root_ppn: 0,
        };
        
        task.context.regs[2] = aligned_sp as u64;
        task.context.mepc = entry as u64;
        
        task
    }

    // [還原] 移除 parent_files 參數
    pub fn new_user(id: usize) -> Self {
        let stack = vec![0u8; STACK_SIZE];
        
        Self {
            id,
            stack,
            context: Context::empty(),
            root_ppn: 0,
        }
    }
}

pub struct Scheduler {
    pub tasks: Vec<Box<Task>>,
    pub current_index: usize,
}

pub static mut SCHEDULER: Option<Scheduler> = None;

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_index: 0,
        }
    }

    pub fn init() {
        unsafe {
            SCHEDULER = Some(Self::new());
        }
    }

    pub fn spawn(&mut self, t: Task) {
        self.tasks.push(Box::new(t));
    }

    pub unsafe fn schedule(&mut self) -> *mut Context {
        if self.tasks.is_empty() {
            panic!("No tasks to schedule!");
        }

        self.current_index = (self.current_index + 1) % self.tasks.len();
        
        let next_task = &mut self.tasks[self.current_index];

        let satp_val = if next_task.root_ppn != 0 {
            (8 << 60) | next_task.root_ppn
        } else {
            let kernel_root = unsafe { KERNEL_PAGE_TABLE as usize };
            (8 << 60) | (kernel_root >> 12)
        };
        
        unsafe {
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
        }

        &mut next_task.context as *mut Context
    }

    pub fn current_task(&mut self) -> &mut Task {
        &mut self.tasks[self.current_index]
    }
}

pub fn get_scheduler() -> &'static mut Scheduler {
    unsafe {
        let ptr = &raw mut SCHEDULER;
        (*ptr).as_mut().unwrap()
    }
}