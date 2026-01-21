
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with User Mode & Syscalls   
-----------------------------------
[OS] Switching to User Mode...
Task 1 [User]: count = 0, addr = 0x80028148
Task 1 [User]: count = 1, addr = 0x80028148
Task 2 [User]: count = 0, addr = 0x8002c148
Task 2 [User]: count = 1, addr = 0x8002c148
Task 1 [User]: count = 2, addr = 0x80028148
Task 2 [User]: count = 2, addr = 0x8002c148
Task 1 [User]: count = 3, addr = 0x80028148
Task 2 [User]: count = 3, addr = 0x8002c148
QEMU: Terminated
```
