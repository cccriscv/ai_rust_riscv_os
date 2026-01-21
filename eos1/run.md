
```
(py310) cccimac@cccimacdeiMac user_app % ./build.sh
     Removed 15 files, 33.1KiB total
   Compiling user_app v0.1.0 (/Users/cccimac/Desktop/ccc/project/ai_rust_riscv_os/user_app)
    Finished `release` profile [optimized] target(s) in 0.81s

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
(py310) cccimac@cccimacdeiMac user_app % cd ../eos1
(py310) cccimac@cccimacdeiMac eos1 % cargo build
   Compiling eos1 v0.1.0 (/Users/cccimac/Desktop/ccc/project/ai_rust_riscv_os/eos1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with Args Support           
-----------------------------------
[Kernel] MMU Enabled.
[OS] Jumping to User Mode...
Shell initialized (Args Enabled).
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
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
exec program.elf Hello World 123
Loading program.elf with args ["program.elf", "Hello", "World", "123"]...
[Kernel] Spawning process with 4 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 4
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "Hello"
[UserApp] argv[2] = "World"
[UserApp] argv[3] = "123"
[Kernel] Process exited code: 0
exec program.elf Hello RISC-V World
Loading program.elf with args ["program.elf", "Hello", "RISC-V", "World"]...
[Kernel] Spawning process with 4 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 4
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "Hello"
[UserApp] argv[2] = "RISC-V"
[UserApp] argv[3] = "World"
[Kernel] Process exited code: 0
exec program.elf 123 12345 1234567 AVeryLongStringWithoutSpaces
Loading program.elf with args ["program.elf", "123", "12345", "1234567", "AVeryLongStringWithoutSpaces"]...
[Kernel] Spawning process with 5 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 5
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "123"
[UserApp] argv[2] = "12345"
[UserApp] argv[3] = "1234567"
[UserApp] argv[4] = "AVeryLongStringWithoutSpaces"
[Kernel] Process exited code: 0
exec program.elf 1 2 3 4 5 6 7 8 9 10
Loading program.elf with args ["program.elf", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"]...
[Kernel] Spawning process with 11 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 11
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "1"
[UserApp] argv[2] = "2"
[UserApp] argv[3] = "3"
[UserApp] argv[4] = "4"
[UserApp] argv[5] = "5"
[UserApp] argv[6] = "6"
[UserApp] argv[7] = "7"
[UserApp] argv[8] = "8"
[UserApp] argv[9] = "9"
[UserApp] argv[10] = "10"
[Kernel] Process exited code: 0
exec program.elf test
Loading program.elf with args ["program.elf", "test"]...
[Kernel] Spawning process with 2 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 2
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "test"
[Kernel] Process exited code: 0
exec program.elf test
Loading program.elf with args ["program.elf", "test"]...
[Kernel] Spawning process with 2 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 2
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "test"
[Kernel] Process exited code: 0
exec program.elf test
Loading program.elf with args ["program.elf", "test"]...
[Kernel] Spawning process with 2 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 2
[UserApp] argv[0] = "program.elf"
[UserApp] argv[1] = "test"
[Kernel] Process exited code: 0
QEMU: Terminated
```
