# ai_rust_riscv_os

我用 Gemini 3 pro 做的 RISCV 處理器之作業系統

## 安裝

必須先安裝 rust 環境 (包含 cargo/rustup) 

```
# 安裝 Rust (如果尚未安裝)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 切換到 Nightly 並安裝必要的組件
rustup default nightly
rustup component add rust-src llvm-tools-preview
rustup target add riscv64gc-unknown-none-elf
cargo install cargo-binutils
```

安裝 qemu-system-riscv64

```
brew install qemu
```

## 編譯與執行

先 git clone 本專案，然後進入專案資料夾後，執行 ./run.sh 指令

```
(py310) cccimac@cccimacdeiMac ai_rust_riscv_os % ./run.sh
     Removed 13 files, 21.0KiB total
   Compiling user_app v0.1.0 (/Users/cccimac/Desktop/ccc/project/ai_rust_riscv_os/user_app)
    Finished `release` profile [optimized] target(s) in 0.65s

target/riscv64gc-unknown-none-elf/release/user_app:     file format elf64-littleriscv

Sections:
Idx Name              Size     VMA              Type
  0                   00000000 0000000000000000 
  1 .text             00000a98 0000000000010000 TEXT
  2 .rodata           000002c8 0000000000010a98 DATA
  3 .eh_frame         000001e0 0000000000010d60 DATA
  4 .comment          00000099 0000000000000000 
  5 .riscv.attributes 00000074 0000000000000000 
  6 .shstrtab         0000003e 0000000000000000 
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/mkfs`
--- SimpleFS Recursive Packer ---
Packing FILE: program.elf
Packing DIR : docs
Packing FILE: note.txt
Packing FILE: secret.txt
Packing FILE: hello.txt
Done! Created disk.img
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
-----------------------------------
   EOS Refactored (v1.0)           
-----------------------------------
[Kernel] Mapping MMIO (PLIC & VirtIO)...
[Kernel] MMU Enabled.
[Kernel] Devices Initialized.
[OS] System Ready. Switching to Shell...
Shell initialized (RW FS).
eos> ls
 - program.elf
 - docs/
 - secret.txt
 - hello.txt
eos> cd docs
Changed directory.
eos> ls
 - note.txt
eos> cat note.txt
I am inside a folder
eos> cd ..
Directory not found.
eos> cd /
Changed directory.
eos> ls
 - program.elf
 - docs/
 - secret.txt
 - hello.txt
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> exec program.elf
Loading program.elf with args ["program.elf"]...
[Kernel] Spawning process with 1 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 1
[UserApp] argv[0] = "program.elf"
[Kernel] Process exited code: 0
ls
 - program.elf
 - docs/
 - secret.txt
 - hello.txt
eos> QEMU: Terminated
```

在 QEMU 要跳出來，必須使用 Ctrl-A-X 