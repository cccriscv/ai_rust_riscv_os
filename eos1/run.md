
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with Round-Robin Scheduler  
-----------------------------------
[Kernel] Heap Allocator initialized.
[Kernel] MMU Enabled.
[Kernel] Tasks spawned.
[OS] Starting Scheduler...
Shell initialized (GC Heap Enabled).
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> exec program.elf
Loading program.elf...
[Kernel] Spawning new process...
[Kernel] ELF loaded.
[Kernel] Process spawned with PID 2
eos> 
[UserApp] Hello, World!
[UserApp] I am running at 0x10000
[UserApp] Calculation: 10 + 20 = 30
[Kernel] Process exited code: 0
ls
 - hello.txt
 - secret.txt
 - program.elf
eos> QEMU: Terminated
```
