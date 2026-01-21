use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

// [修正 1] 定義常數，避免 runtime 去讀取 static mut 的長度
const HEAP_SIZE: usize = 64 * 1024;

static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut HEAP_INDEX: usize = 0;

pub struct SimpleAllocator;

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        
        // [修正 2] 使用 &raw mut 取得原生指標，避免建立 Rust Reference
        // 這裡直接將陣列的 raw pointer 轉為 usize 位址
        let start_addr = &raw mut HEAP_MEMORY as usize;
        
        // 讀取目前的 index
        // 因為是 usize (Copy type)，直接讀取是允許的，或者也可以用 &raw mut 讀取
        // 這裡為了保險起見，我們讀取它的值
        let mut index = HEAP_INDEX;
        
        let mut current_addr = start_addr + index;

        // 計算對齊
        let remainder = current_addr % align;
        if remainder != 0 {
            let padding = align - remainder;
            index += padding;
            current_addr += padding;
        }

        // [修正 3] 直接使用常數 HEAP_SIZE，不要呼叫 HEAP_MEMORY.len()
        if index + size > HEAP_SIZE {
            return null_mut();
        }

        // 更新 index
        HEAP_INDEX = index + size;
        
        current_addr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // 簡單實作，不回收
    }
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator;