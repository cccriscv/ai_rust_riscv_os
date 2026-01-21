use alloc::string::String;
use alloc::vec::Vec;

pub struct File {
    pub name: &'static str,
    pub data: &'static [u8],
}

static FILES: &[File] = &[
    File {
        name: "hello.txt",
        data: include_bytes!("../disk/hello.txt"),
    },
    File {
        name: "secret.txt",
        data: include_bytes!("../disk/secret.txt"),
    },
    // [新增] 加入 ELF 執行檔
    File {
        name: "program.elf",
        data: include_bytes!("../disk/program.elf"),
    },
];

pub fn get_file_content(name: &str) -> Option<&'static [u8]> {
    for file in FILES {
        if file.name == name {
            return Some(file.data);
        }
    }
    None
}

pub fn list_files() -> Vec<String> {
    let mut list = Vec::new();
    for file in FILES {
        list.push(String::from(file.name));
    }
    list
}