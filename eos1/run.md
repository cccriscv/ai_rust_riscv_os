
```
(py310) cccimac@cccimacdeiMac user_app % ./build.sh 
     Removed 6 files, 4.0KiB total
   Compiling user_app v0.1.0 (/Users/cccimac/Desktop/ccc/project/ai_rust_riscv_os/user_app)
    Finished `release` profile [optimized] target(s) in 0.73s

target/riscv64gc-unknown-none-elf/release/user_app:     file format elf64-littleriscv

Sections:
Idx Name              Size     VMA              Type
  0                   00000000 0000000000000000 
  1 .text             000005f6 0000000080200000 TEXT
  2 .rodata           000001a8 00000000802005f8 DATA
  3 .eh_frame         00000134 00000000802007a0 DATA
  4 .comment          00000099 0000000000000000 
  5 .riscv.attributes 00000074 0000000000000000 
  6 .shstrtab         0000003e 0000000000000000 
(py310) cccimac@cccimacdeiMac user_app % cd ../eos1
(py310) cccimac@cccimacdeiMac eos1 % cargo build
...
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with ELF Loader (M-Mode)    
-----------------------------------
[OS] User Mode initialized.
Shell initialized. Type 'exec program.elf' to run.
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> exec program.elf
Loading program.elf...
[Kernel] Loading ELF...
[Kernel] Jumping to 80200000

[UserApp] Hello, World!
[UserApp] I am running at 0x80200000
[UserApp] Calculation: 10 + 20 = 30

[Trap caught] mcause=2, mepc=80200084
User App terminated. Rebooting shell...
Shell initialized. Type 'exec program.elf' to run.
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> QEMU: Terminated
```
