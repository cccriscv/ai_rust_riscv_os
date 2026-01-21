use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

// [修正] 加大 Heap 到 1MB
const HEAP_SIZE: usize = 1024 * 1024; 

static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut HEAP_INDEX: usize = 0;

pub struct SimpleAllocator;

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        
        let start_addr = &raw mut HEAP_MEMORY as usize;
        let mut index = unsafe { HEAP_INDEX };
        
        let mut current_addr = start_addr + index;

        let remainder = current_addr % align;
        if remainder != 0 {
            let padding = align - remainder;
            index += padding;
            current_addr += padding;
        }

        if index + size > HEAP_SIZE {
            return null_mut();
        }

        unsafe {
            HEAP_INDEX = index + size;
        }
        
        current_addr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // 不回收
    }
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator;