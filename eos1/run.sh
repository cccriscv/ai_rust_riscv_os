#!/bin/bash

# 編譯
cargo build

# 執行 QEMU
# 新增參數：
# -drive file=disk.img,if=none,format=raw,id=x0 
# -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -kernel target/riscv64gc-unknown-none-elf/debug/eos1 \
    -drive file=disk.img,if=none,format=raw,id=x0 \
    -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0