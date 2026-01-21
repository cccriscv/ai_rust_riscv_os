// src/mm/frame.rs

// [修正 1] 移除未使用的 import
// use core::ptr::null_mut;

// [修正 2] 加上 unsafe 關鍵字
unsafe extern "C" {
    fn ekernel();
}

const RAM_END: usize = 0x8800_0000;

static mut NEXT_PFN: usize = 0;

pub fn init() {
    unsafe {
        NEXT_PFN = ekernel as usize;
        if NEXT_PFN % 4096 != 0 {
            NEXT_PFN += 4096 - (NEXT_PFN % 4096);
        }
    }
}

pub fn alloc_frame() -> usize {
    unsafe {
        let paddr = NEXT_PFN;
        let next_paddr = paddr + 4096;

        if next_paddr >= RAM_END {
            return 0; 
        }

        NEXT_PFN = next_paddr;
        
        core::ptr::write_bytes(paddr as *mut u8, 0, 4096);
        
        paddr
    }
}