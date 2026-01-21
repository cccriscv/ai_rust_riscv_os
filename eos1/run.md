
```
(py310) cccimac@cccimacdeiMac eos1-v0.8-ramdisk % ./run.sh
-----------------------------------
   EOS with RAM Filesystem         
-----------------------------------
[OS] Filesystem mounted. Jumping to User Mode...
Shell initialized. Type 'help' for commands.
eos> help
Commands: help, ls, cat <file>, panic
eos> help
Commands: help, ls, cat <file>, panic
eos> ls
 - hello.txt
 - secret.txt
eos> cat hello.txt
--- begin hello.txt ---
Hello! This is a text file stored in the Kernel.
Rust OS is fun!
--- end ---
eos> cat secret.txt
--- begin secret.txt ---
Top Secret Data: The answer is 42.
--- end ---
eos> panic
Crashing...

[Segmentation Fault]
Shell initialized. Type 'help' for commands.
eos> help
Commands: help, ls, cat <file>, panic
eos> QEMU: Terminated
```
