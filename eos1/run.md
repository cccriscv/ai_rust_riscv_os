
```
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh > run.txt
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh          
-----------------------------------
   EOS with Process Isolation      
-----------------------------------
[Kernel] Frame Allocator initialized.
[Kernel] MMU Enabled.
[OS] Jumping to User Mode...
Shell initialized (Isolated).
eos> exec program.elf
Loading program.elf...
[Kernel] Executing new process...
[Kernel] ELF loaded. Switching SATP.

[UserApp] Hello, World!
[UserApp] I am running at 0x10000
[UserApp] Calculation: 10 + 20 = 30
[Kernel] User App exited with code: 0
Rebooting shell...
Shell initialized (Isolated).
eos> QEMU: Terminated
```
