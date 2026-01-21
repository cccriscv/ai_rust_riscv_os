use alloc::vec::Vec;
use alloc::string::String;
use crate::virtio;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Superblock {
    magic: u32,
    file_count: u32,
    _padding: [u8; 504],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEntry {
    name: [u8; 32],
    start_sector: u32,
    size: u32,
    _padding: [u8; 24],
}

impl Default for DirEntry {
    fn default() -> Self {
        Self { name: [0; 32], start_sector: 0, size: 0, _padding: [0; 24] }
    }
}

// 讀取檔案內容
pub fn get_file_content(name: &str) -> Option<Vec<u8>> {
    let sb_data = virtio::read_disk(0);
    let sb = unsafe { &*(sb_data.as_ptr() as *const Superblock) };
    if sb.magic != 0x53465331 { return None; }

    let dir_data = virtio::read_disk(1);
    let entries = unsafe { core::slice::from_raw_parts(dir_data.as_ptr() as *const DirEntry, 8) };

    for i in 0..sb.file_count as usize {
        if i >= 8 { break; }
        let entry = &entries[i];
        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        let entry_name = core::str::from_utf8(&entry.name[0..name_end]).unwrap_or("");

        if entry_name == name {
            let mut content = Vec::new();
            let mut current_sec = entry.start_sector;
            let mut remaining = entry.size;

            while remaining > 0 {
                let sector_data = virtio::read_disk(current_sec as u64);
                let copy_len = core::cmp::min(remaining as usize, 512);
                content.extend_from_slice(&sector_data[0..copy_len]);
                remaining -= copy_len as u32;
                current_sec += 1;
            }
            return Some(content);
        }
    }
    None
}

// 列出檔案
pub fn list_files() -> Vec<String> {
    let mut list = Vec::new();
    let sb_data = virtio::read_disk(0);
    let sb = unsafe { &*(sb_data.as_ptr() as *const Superblock) };

    if sb.magic != 0x53465331 {
        list.push(String::from("INVALID_FS"));
        return list;
    }

    let dir_data = virtio::read_disk(1);
    let entries = unsafe { core::slice::from_raw_parts(dir_data.as_ptr() as *const DirEntry, 8) };

    for i in 0..sb.file_count as usize {
        if i >= 8 { break; }
        let entry = &entries[i];
        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        if let Ok(name) = core::str::from_utf8(&entry.name[0..name_end]) {
            list.push(String::from(name));
        }
    }
    list
}

// [新增] 寫入檔案 (如果檔名存在則覆寫，不存在則新增)
// 回傳: 0=成功, -1=錯誤, -2=磁碟滿
pub fn write_file(name: &str, data: &[u8]) -> isize {
    // 1. 讀取 Superblock 和 Directory
    let mut sb_buf = virtio::read_disk(0); // 複製一份 Buffer 以便修改
    let sb = unsafe { &mut *(sb_buf.as_mut_ptr() as *mut Superblock) };
    
    if sb.magic != 0x53465331 { return -1; }

    let mut dir_buf = virtio::read_disk(1);
    let entries = unsafe { core::slice::from_raw_parts_mut(dir_buf.as_mut_ptr() as *mut DirEntry, 8) };

    // 2. 檢查檔案是否已存在
    let mut target_index = None;
    let mut max_sector_used = 10; // Data Area starts at 10

    for i in 0..sb.file_count as usize {
        if i >= 8 { break; }
        let entry = &entries[i];
        
        // 算出目前用到的最大 Sector，以便 append 新檔案
        let sectors_used = (entry.size + 511) / 512;
        let end_sector = entry.start_sector + sectors_used;
        if end_sector > max_sector_used {
            max_sector_used = end_sector;
        }

        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        let entry_name = core::str::from_utf8(&entry.name[0..name_end]).unwrap_or("");
        
        if entry_name == name {
            target_index = Some(i);
        }
    }

    // 3. 決定寫入位置
    let index;
    let start_sector;

    if let Some(idx) = target_index {
        // 覆寫模式 (簡化：我們不回收舊空間，直接在最後面寫新的，並更新指標)
        // 這會導致舊資料變成垃圾空間 (Leak)，但在 SimpleFS 是可接受的權衡
        index = idx;
        start_sector = max_sector_used;
    } else {
        // 新增模式
        if sb.file_count >= 8 { return -2; } // Directory Full
        index = sb.file_count as usize;
        start_sector = max_sector_used;
        sb.file_count += 1;
    }

    // 4. 寫入檔案內容到 Data Area
    let mut current_sector = start_sector;
    let mut remaining = data.len();
    let mut offset = 0;

    while remaining > 0 {
        let copy_len = core::cmp::min(remaining, 512);
        let mut sector_data = [0u8; 512];
        
        sector_data[0..copy_len].copy_from_slice(&data[offset..offset+copy_len]);
        virtio::write_disk(current_sector as u64, &sector_data);
        
        remaining -= copy_len;
        offset += copy_len;
        current_sector += 1;
    }

    // 5. 更新 Directory Entry
    let entry = &mut entries[index];
    let name_bytes = name.as_bytes();
    let copy_len = core::cmp::min(name_bytes.len(), 32);
    
    // 清空舊名並寫入新名
    entry.name = [0; 32];
    entry.name[0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
    entry.start_sector = start_sector;
    entry.size = data.len() as u32;

    // 6. 寫回 Directory 和 Superblock
    // 注意：write_disk 吃的是 &[u8]，我們可以直接傳 Buffer
    virtio::write_disk(1, &dir_buf); // Directory
    virtio::write_disk(0, &sb_buf);  // Superblock

    0 // Success
}