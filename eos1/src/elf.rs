use core::mem::size_of;

// ELF 64-bit Header (檔案頭)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ElfHeader {
    pub magic: [u8; 4],     // 0x7F 'E' 'L' 'F'
    pub class: u8,          // 2 = 64-bit
    pub endian: u8,         // 1 = Little Endian
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub pad: [u8; 7],
    pub type_: u16,         // 2 = Executable
    pub machine: u16,       // 0xF3 = RISC-V
    pub version2: u32,
    pub entry: u64,         // 程式進入點
    pub phoff: u64,         // Program Header Table Offset
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,     // Program Header Entry Size
    pub phnum: u16,         // Program Header Count
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

// Program Header (描述記憶體區段)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProgramHeader {
    pub type_: u32,         // 1 = LOAD
    pub flags: u32,         // R/W/X
    pub offset: u64,        // 檔案中的位移
    pub vaddr: u64,         // 記憶體中的虛擬位址
    pub paddr: u64,
    pub filesz: u64,        // 檔案中的大小
    pub memsz: u64,         // 記憶體中的大小 (包含 BSS)
    pub align: u64,
}

/// 解析並載入 ELF 檔案
/// 回傳: Option<Entry Point Address>
pub unsafe fn load_elf(data: &[u8]) -> Option<u64> {
    // 1. 基本長度檢查
    if data.len() < size_of::<ElfHeader>() {
        return None;
    }

    // 取得 Header
    let header = &*(data.as_ptr() as *const ElfHeader);

    // 2. 驗證 Magic Number: 0x7F, 'E', 'L', 'F'
    if header.magic != [0x7f, 0x45, 0x4c, 0x46] {
        return None;
    }

    // 3. 驗證架構: RISC-V (0xF3)
    if header.machine != 0xF3 {
        return None;
    }

    // 4. 遍歷 Program Headers
    let ph_table_ptr = data.as_ptr().add(header.phoff as usize);
    
    for i in 0..header.phnum {
        let ph_ptr = ph_table_ptr.add((i as usize) * (header.phentsize as usize));
        let ph = &*(ph_ptr as *const ProgramHeader);

        // 只處理 LOAD 類型的區段 (Type = 1)
        if ph.type_ == 1 {
            // [關鍵安全檢查]
            // 如果 user_app 編譯時 linker script 沒生效，vaddr 預設會是 0x10000 左右
            // 核心如果試圖寫入那裡，就會觸發 mcause=7 崩潰
            // 我們只允許載入到 RAM 區域 (0x80000000 起始)
            if ph.vaddr < 0x80000000 {
                return None; // 拒絕載入非法位址的程式
            }

            let dest = ph.vaddr as *mut u8;
            let src = data.as_ptr().add(ph.offset as usize);
            
            // 防止 buffer overflow 讀取
            if ph.offset + ph.filesz > data.len() as u64 {
                return None;
            }

            // 拷貝程式碼與資料 (Memcpy)
            if ph.filesz > 0 {
                core::ptr::copy_nonoverlapping(src, dest, ph.filesz as usize);
            }

            // 處理 BSS (未初始化的變數，補 0)
            if ph.memsz > ph.filesz {
                let zero_start = dest.add(ph.filesz as usize);
                let zero_len = (ph.memsz - ph.filesz) as usize;
                core::ptr::write_bytes(zero_start, 0, zero_len);
            }
        }
    }

    // 回傳程式進入點
    Some(header.entry)
}