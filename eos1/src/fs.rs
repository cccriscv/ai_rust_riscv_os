use alloc::string::String;
use alloc::vec::Vec;

pub struct File {
    pub name: &'static str,
    pub data: &'static [u8],
}

// 這裡使用 include_bytes! 將檔案內容直接編譯進執行檔
// 這就是最簡單的 RAM Disk
static FILES: &[File] = &[
    File {
        name: "hello.txt",
        data: include_bytes!("../disk/hello.txt"),
    },
    File {
        name: "secret.txt",
        data: include_bytes!("../disk/secret.txt"),
    },
];

/// 根據檔名尋找檔案內容，回傳 Option<&[u8]>
pub fn get_file_content(name: &str) -> Option<&'static [u8]> {
    for file in FILES {
        if file.name == name {
            return Some(file.data);
        }
    }
    None
}

/// 列出所有檔名
pub fn list_files() -> Vec<String> {
    let mut list = Vec::new();
    for file in FILES {
        list.push(String::from(file.name));
    }
    list
}