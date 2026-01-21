use core::mem::size_of;
// [修正] 引入 PageTable 結構
use crate::mm::page_table::{map, translate, PageTable, PTE_R, PTE_W, PTE_X, PTE_U};
use crate::mm::frame::alloc_frame;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    pub magic: [u8; 4],
    pub class: u8,
    pub endian: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub pad: [u8; 7],
    pub type_: u16,
    pub machine: u16,
    pub version2: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader {
    pub type_: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

/// 解析 ELF 並載入到指定的 Page Table 中
pub unsafe fn load_elf(data: &[u8], page_table: &mut PageTable) -> Option<u64> {
    if data.len() < size_of::<ElfHeader>() { return None; }
    let header = unsafe { &*(data.as_ptr() as *const ElfHeader) };

    if header.magic != [0x7f, 0x45, 0x4c, 0x46] || header.machine != 0xF3 {
        return None;
    }

    let ph_table_ptr = unsafe { data.as_ptr().add(header.phoff as usize) };
    
    // [修改] 使用傳入的 page_table 作為映射目標
    let root = page_table;

    for i in 0..header.phnum {
        let ph_ptr = unsafe { ph_table_ptr.add((i as usize) * (header.phentsize as usize)) };
        let ph = unsafe { &*(ph_ptr as *const ProgramHeader) };

        if ph.type_ == 1 { // LOAD Segment
            let start_vpn = ph.vaddr >> 12;
            let end_vpn = (ph.vaddr + ph.memsz + 4095) >> 12;

            for vpn in start_vpn..end_vpn {
                let page_vaddr = (vpn << 12) as usize;
                
                // 檢查是否已映射 (避免同一頁面重複分配導致資料覆蓋)
                let mut paddr = unsafe { translate(root, page_vaddr).unwrap_or(0) };

                if paddr == 0 {
                    // 分配新的實體頁面
                    paddr = alloc_frame();
                    if paddr == 0 { return None; } 
                    
                    unsafe {
                        map(root, page_vaddr, paddr, PTE_U | PTE_R | PTE_W | PTE_X);
                    }
                }

                // 計算頁內偏移與寫入位置
                // 寫入時使用 paddr (實體位址)，因為 M-Mode 核心不經過 MMU
                let page_offset = if vpn == start_vpn { (ph.vaddr % 4096) as usize } else { 0 };
                let dest_ptr = (paddr + page_offset) as *mut u8;
                let page_remaining = 4096 - page_offset;
                
                let processed_len = (page_vaddr + page_offset) - (ph.vaddr as usize);
                
                if (processed_len as u64) < ph.filesz {
                    let src_ptr = unsafe { data.as_ptr().add((ph.offset as usize) + processed_len) };
                    let copy_len = core::cmp::min(page_remaining, (ph.filesz as usize) - processed_len);
                    
                    unsafe { core::ptr::copy_nonoverlapping(src_ptr, dest_ptr, copy_len); }
                }
                // BSS (補 0) 的部分由 alloc_frame 初始化時處理，這裡省略
            }
        }
    }

    // 確保指令寫入可見
    unsafe { core::arch::asm!("fence.i"); }
    
    Some(header.entry)
}