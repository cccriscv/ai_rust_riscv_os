
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
-----------------------------------
   EOS Refactored (v1.0)           
-----------------------------------
[Kernel] Mapping MMIO (PLIC & VirtIO)...
[Kernel] MMU Enabled.
[Kernel] Devices Initialized.
[OS] System Ready. Switching to Shell...
Shell initialized (Refactored).
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> exec program.elf
Loading program.elf...
[Kernel] Spawning process with 1 args...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Started!
[UserApp] argc = 1
[UserApp] argv[0] = "program.elf"
[Kernel] Process exited code: 0
ls
 - hello.txt
 - secret.txt
 - program.elf
eos> QEMU: Terminated
```
