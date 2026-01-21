use std::fs;
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::mem::size_of;
use std::path::Path;

const SOURCE_DIR: &str = "fs_root";
const TARGET_IMG: &str = "disk.img";
const DISK_SIZE: u64 = 32 * 1024 * 1024;

// 0=File, 1=Directory
const TYPE_FILE: u8 = 0;
const TYPE_DIR: u8 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Superblock {
    magic: u32,
    file_count: u32, // Root dir file count
    _padding: [u8; 504],
}

impl Default for Superblock {
    fn default() -> Self {
        Self { magic: 0, file_count: 0, _padding: [0; 504] }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEntry {
    name: [u8; 32],
    start_sector: u32,
    size: u32,
    file_type: u8,      // [新增] 類型
    _padding: [u8; 23], // [修改] 剩餘填充
}

impl Default for DirEntry {
    fn default() -> Self {
        Self { name: [0; 32], start_sector: 0, size: 0, file_type: 0, _padding: [0; 23] }
    }
}

// 全域變數：追蹤目前寫到哪個 Sector
static mut CURRENT_SECTOR: u32 = 10;

fn main() -> std::io::Result<()> {
    println!("--- SimpleFS Recursive Packer ---");

    // 1. 初始化磁碟
    let mut disk = File::create(TARGET_IMG)?;
    disk.set_len(DISK_SIZE)?;

    // 2. 遞迴處理根目錄，取得根目錄的內容 (Byte Array)
    // 根目錄的內容其實就是 Directory Table
    let (root_entries_bytes, file_count) = process_directory(Path::new(SOURCE_DIR), &mut disk)?;

    // 3. 寫入 Superblock (Sector 0)
    let mut sb = Superblock::default();
    sb.magic = 0x53465331;
    sb.file_count = file_count;
    
    let sb_bytes = unsafe {
        std::slice::from_raw_parts(&sb as *const _ as *const u8, size_of::<Superblock>())
    };
    disk.seek(SeekFrom::Start(0))?;
    disk.write_all(sb_bytes)?;

    // 4. 寫入 Root Directory Table (Sector 1)
    // 注意：SimpleFS 規定 Sector 1 是根目錄表
    disk.seek(SeekFrom::Start(512))?;
    if root_entries_bytes.len() > 512 {
        panic!("Root directory too large (> 8 files)!");
    }
    disk.write_all(&root_entries_bytes)?;

    println!("Done! Created {}", TARGET_IMG);
    Ok(())
}

// 遞迴函數：處理一個資料夾，回傳該資料夾的 Directory Table (bytes) 和檔案數
fn process_directory(dir_path: &Path, disk: &mut File) -> std::io::Result<(Vec<u8>, u32)> {
    let mut entries = Vec::new();
    let mut count = 0;

    let paths = fs::read_dir(dir_path)?;

    for entry in paths {
        let entry = entry?;
        let path = entry.path();
        let name_str = path.file_name().unwrap().to_str().unwrap();
        
        // 忽略隱藏檔
        if name_str.starts_with('.') { continue; }

        let mut dir_entry = DirEntry::default();
        
        // 複製檔名
        let name_bytes = name_str.as_bytes();
        if name_bytes.len() > 32 { panic!("Name too long: {}", name_str); }
        dir_entry.name[..name_bytes.len()].copy_from_slice(name_bytes);

        if path.is_dir() {
            println!("Packing DIR : {}", name_str);
            // [遞迴] 處理子目錄
            // 子目錄的「內容」就是它裡面的 Directory Table
            let (subdir_table_bytes, _) = process_directory(&path, disk)?;
            
            // 將這個 Table 寫入 Data Area，就像寫普通檔案一樣
            let start_sec = unsafe { CURRENT_SECTOR };
            let size = subdir_table_bytes.len() as u32;
            
            disk.seek(SeekFrom::Start(start_sec as u64 * 512))?;
            disk.write_all(&subdir_table_bytes)?;
            
            // 計算佔用 Sector
            let sectors = (size + 511) / 512;
            unsafe { CURRENT_SECTOR += sectors; }

            dir_entry.file_type = TYPE_DIR;
            dir_entry.start_sector = start_sec;
            dir_entry.size = size;

        } else {
            println!("Packing FILE: {}", name_str);
            // 處理普通檔案
            let mut file = File::open(&path)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;

            let start_sec = unsafe { CURRENT_SECTOR };
            let size = content.len() as u32;

            disk.seek(SeekFrom::Start(start_sec as u64 * 512))?;
            disk.write_all(&content)?;

            let sectors = (size + 511) / 512;
            unsafe { CURRENT_SECTOR += sectors; }

            dir_entry.file_type = TYPE_FILE;
            dir_entry.start_sector = start_sec;
            dir_entry.size = size;
        }

        entries.push(dir_entry);
        count += 1;
    }

    // 將 entries 序列化成 bytes
    let mut table_bytes = Vec::new();
    for e in entries {
        let bytes = unsafe {
            std::slice::from_raw_parts(&e as *const _ as *const u8, size_of::<DirEntry>())
        };
        table_bytes.extend_from_slice(bytes);
    }
    
    // 補齊到 512 bytes (一個 Sector) 的倍數 (Optional, 視需求)
    // 這裡我們不補齊，直接回傳實際大小，讓上層決定怎麼寫
    
    Ok((table_bytes, count))
}