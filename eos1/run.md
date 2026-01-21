
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh 
-----------------------------------
   EOS with RAM Filesystem         
-----------------------------------
[OS] Filesystem mounted. Jumping to User Mode...
Shell initialized. Type 'help' for commands.
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
eos> QEMU: Terminated
```
