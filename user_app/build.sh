cargo clean
cargo build --release
rust-objdump -h target/riscv64gc-unknown-none-elf/release/user_app
cp target/riscv64gc-unknown-none-elf/release/user_app ../eos1/disk/program.elf
# [修正 2] 加上 --target 參數
# cargo build --release --target riscv64gc-unknown-none-elf
