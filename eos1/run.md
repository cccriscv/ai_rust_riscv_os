
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
   Compiling eos1 v0.1.0 (/Users/cccimac/Desktop/ccc/cpu2os/02-系統程式/_rust/os/eos1/eos1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.84s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh   
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
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> ls docs
 - program.elf
 - docs/
 - secret.txt
 - hello.txt
eos> cd docs
Changed directory.
eos> ls
 - note.txt
eos> cd /
Changed directory.
eos> ls
 - program.elf
 - docs/
 - secret.txt
 - hello.txt
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
