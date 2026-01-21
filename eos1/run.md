
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build       
   Compiling eos1 v0.1.0 (/Users/cccimac/Desktop/ccc/cpu2os/02-系統程式/_rust/os/eos1/eos1)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.75s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh          
-----------------------------------
   EOS with PLIC Interrupts        
-----------------------------------
[Kernel] Frame Allocator initialized.
[Kernel] Mapping PLIC...
[Kernel] MMU Enabled.
[Kernel] PLIC Initialized.
[OS] Jumping to User Mode...
Shell initialized (Interrupt Driven).
eos> help
ls, cat <file>, exec <file>
eos> exec program.elf
Loading program.elf...
[Kernel] Executing new process...
[Kernel] ELF loaded. Switching SATP.

[UserApp] Hello, World!
[UserApp] I am running at 0x10000
[UserApp] Calculation: 10 + 20 = 30
[Kernel] User App exited with code: 0
Rebooting shell...
Shell initialized (Interrupt Driven).
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> cat secret.txt
Top Secret Data: The answer is 42.
eos> cat hello.txt
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
eos> ls
 - hello.txt
 - secret.txt
 - program.elf
```
