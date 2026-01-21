
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
   Compiling eos1 v0.1.0 (/Users/cccimac/Desktop/ccc/cpu2os/02-系統程式/_rust/os/eos1/eos1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.78s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with Round-Robin Scheduler  
-----------------------------------
[Kernel] MMU Enabled.
[Kernel] Tasks spawned.
[OS] Starting Scheduler...
Shell initialized (Scheduler V1).
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> exec program.elf
Loading program.elf...
[Kernel] Spawning new process...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Hello, World!
[UserApp] I am running at 0x10000
[UserApp] Calculation: 10 + 20 = 30
ls
 - hello.txt
 - secret.txt
 - program.elf
eos> QEMU: Terminated
```
