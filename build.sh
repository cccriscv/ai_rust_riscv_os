set -x
# 1. 進入 user_app 編譯
cd user_app
cargo clean
cargo build --release

# 2. 複製執行檔到 mkfs 的來源目錄
# 注意：你的目錄結構可能不同，請根據實際情況調整 ../
cp target/riscv64gc-unknown-none-elf/release/program ../mkfs/fs_root/program.elf
cp target/riscv64gc-unknown-none-elf/release/ls ../mkfs/fs_root/ls
cp target/riscv64gc-unknown-none-elf/release/cat ../mkfs/fs_root/cat

# 3. 重新打包磁碟
cd ../mkfs
cargo run
# 複製磁碟到 Kernel 目錄
cp disk.img ../eos1/

# 4. 執行 Kernel
cd ../eos1
./run.sh