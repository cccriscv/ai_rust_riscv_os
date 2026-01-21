
```
(py310) cccimac@cccimacdeiMac eos1-v0.7-shell % ./run.sh
-----------------------------------
   EOS Interactive Shell           
-----------------------------------
[OS] User Mode initialized.
Shell initialized. Type 'help' for commands.
eos> help
Available commands:
  help  - Show this message
  hello - Say hello
  clear - Clear command buffer
  panic - Test kernel panic
eos> hello
Hello from User Mode!
eos> panic
Attempting to crash...

[PANIC] panicked at src/main.rs:127:34:
null pointer dereference occurred
QEMU: Terminated
```
