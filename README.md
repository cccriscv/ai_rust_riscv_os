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

## 編譯

先 git clone 本專案，然後進入專案資料夾後，執行下列指令

```
cargo build
```

## 執行

```
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
Hello, RISC-V OS!
QEMU: Terminated
```

在 QEMU 要跳出來，必須使用 Ctrl-A-X 