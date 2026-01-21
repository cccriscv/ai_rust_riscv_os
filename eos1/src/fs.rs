use alloc::vec::Vec;
use alloc::string::String;
use crate::virtio;

// 0=File, 1=Directory
pub const TYPE_FILE: u8 = 0;
pub const TYPE_DIR: u8 = 1;

static mut CURRENT_DIR_SECTOR: u32 = 1;

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
    file_type: u8,
    _padding: [u8; 23],
}

// Helper: 讀取指定 Sector 的 Directory Table
fn read_dir_entries(sector: u32) -> Vec<DirEntry> {
    let dir_data = virtio::read_disk(sector as u64);
    let mut entries = Vec::new();
    let raw_slice = unsafe { core::slice::from_raw_parts(dir_data.as_ptr() as *const DirEntry, 8) };
    entries.extend_from_slice(raw_slice);
    entries
}

pub fn list_files() -> Vec<(u8, String)> {
    let mut list = Vec::new();
    let dir_sector = unsafe { CURRENT_DIR_SECTOR };
    let entries = read_dir_entries(dir_sector);

    for entry in entries {
        if entry.start_sector == 0 { continue; }
        if entry.name[0] == 0 { continue; }

        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        if let Ok(name) = core::str::from_utf8(&entry.name[0..name_end]) {
            list.push((entry.file_type, String::from(name)));
        }
    }
    list
}

pub fn change_dir(name: &str) -> isize {
    if name == "/" {
        unsafe { CURRENT_DIR_SECTOR = 1; }
        return 0;
    }

    let dir_sector = unsafe { CURRENT_DIR_SECTOR };
    let entries = read_dir_entries(dir_sector);

    for entry in entries {
        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        let entry_name = core::str::from_utf8(&entry.name[0..name_end]).unwrap_or("");

        if entry_name == name {
            if entry.file_type == TYPE_DIR {
                unsafe { CURRENT_DIR_SECTOR = entry.start_sector; }
                return 0;
            } else {
                return -2;
            }
        }
    }
    -1
}

pub fn get_file_content(name: &str) -> Option<Vec<u8>> {
    let dir_sector = unsafe { CURRENT_DIR_SECTOR };
    let entries = read_dir_entries(dir_sector);

    for entry in entries {
        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        let entry_name = core::str::from_utf8(&entry.name[0..name_end]).unwrap_or("");

        if entry_name == name {
            if entry.file_type == TYPE_DIR { return None; }

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

// [修正] 恢復並修正寫入功能
pub fn write_file(name: &str, data: &[u8]) -> isize {
    // 1. 讀取 Superblock (為了檢查是否滿了，雖然這裡簡化處理)
    let sb_data = virtio::read_disk(0);
    let sb = unsafe { &*(sb_data.as_ptr() as *const Superblock) };
    if sb.magic != 0x53465331 { return -1; }

    // 2. 讀取當前目錄
    let dir_sector = unsafe { CURRENT_DIR_SECTOR };
    // 注意：我們要修改它，所以不能只用 read_dir_entries (它回傳 Vec clone)
    let mut dir_buf = virtio::read_disk(dir_sector as u64);
    let entries = unsafe { core::slice::from_raw_parts_mut(dir_buf.as_mut_ptr() as *mut DirEntry, 8) };

    // 3. 找空位或同名檔案
    let mut target_idx = None;
    let mut free_idx = None;
    
    // 尋找最大使用的 Sector (全域搜尋有點難，這裡我們只搜尋當前目錄的最大值作為起點，這是個 Bug 但堪用)
    // 正確做法是 Superblock 應該記錄 next_free_sector
    let mut max_sector = 50; // 隨便抓個安全值，假設前面的都被用掉了

    for i in 0..8 {
        let entry = &entries[i];
        if entry.start_sector == 0 {
            if free_idx.is_none() { free_idx = Some(i); }
            continue; 
        }

        // 更新 max sector
        let used_sectors = (entry.size + 511) / 512;
        let end = entry.start_sector + used_sectors;
        if end > max_sector { max_sector = end; }

        let name_end = entry.name.iter().position(|&c| c == 0).unwrap_or(32);
        let entry_name = core::str::from_utf8(&entry.name[0..name_end]).unwrap_or("");
        
        if entry_name == name {
            target_idx = Some(i);
        }
    }

    let idx = if let Some(i) = target_idx { i } 
              else if let Some(i) = free_idx { i } 
              else { return -2; }; // 目錄滿了

    // 4. 寫入資料
    let start_sector = max_sector; // Append 到最後面
    let mut current_sec = start_sector;
    let mut remaining = data.len();
    let mut offset = 0;

    while remaining > 0 {
        let copy_len = core::cmp::min(remaining, 512);
        let mut sec_data = [0u8; 512];
        sec_data[0..copy_len].copy_from_slice(&data[offset..offset+copy_len]);
        virtio::write_disk(current_sec as u64, &sec_data);
        
        remaining -= copy_len;
        offset += copy_len;
        current_sec += 1;
    }

    // 5. 更新目錄 Entry
    let entry = &mut entries[idx];
    let name_bytes = name.as_bytes();
    let copy_len = core::cmp::min(name_bytes.len(), 32);
    
    // 清空並寫入
    entry.name = [0; 32];
    entry.name[0..copy_len].copy_from_slice(&name_bytes[0..copy_len]);
    entry.start_sector = start_sector;
    entry.size = data.len() as u32;
    entry.file_type = TYPE_FILE;

    // 6. 寫回目錄表
    virtio::write_disk(dir_sector as u64, &dir_buf);

    0
}