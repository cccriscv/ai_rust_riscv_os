
```
(py310) cccimac@cccimacdeiMac eos1 % cargo build
warning[E0133]: use of mutable static is unsafe and requires unsafe block
  --> src/heap.rs:24:25
   |
24 |         let mut index = HEAP_INDEX;
   |                         ^^^^^^^^^^ use of mutable static
   |
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
   = note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior
note: an unsafe function restricts its caller, but its body is safe by default
  --> src/heap.rs:13:5
   |
13 |     unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: `#[warn(unsafe_op_in_unsafe_fn)]` (part of `#[warn(rust_2024_compatibility)]`) on by default

warning[E0133]: use of mutable static is unsafe and requires unsafe block
  --> src/heap.rs:42:9
   |
42 |         HEAP_INDEX = index + size;
   |         ^^^^^^^^^^ use of mutable static
   |
   = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-op-in-unsafe-fn.html>
   = note: mutable statics can be mutated by multiple threads: aliasing violations or data races will cause undefined behavior

For more information about this error, try `rustc --explain E0133`.
warning: `eos1` (bin "eos1") generated 2 warnings (run `cargo fix --bin "eos1" -p eos1` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
(py310) cccimac@cccimacdeiMac eos1 % ./run.sh
-----------------------------------
   EOS with Heap & User Mode       
-----------------------------------
[OS] Heap initialized. Jumping to User Mode...
Task 1 [Heap]: Box=999, Hist=[0]
Task 1 [Heap]: Box=999, Hist=[0, 1]
Task 2 [User]: count = 0
Task 2 [User]: count = 1
Task 1 [Heap]: Box=999, Hist=[0, 1, 2]
Task 2 [User]: count = 2
Task 1 [Heap]: Box=999, Hist=[0, 1, 2, 3]
Task 2 [User]: count = 3
Task 1 [Heap]: Box=999, Hist=[0, 1, 2, 3, 4]
Task 2 [User]: count = 4
Task 1 [Heap]: Box=999, Hist=[1, 2, 3, 4, 5]
Task 2 [User]: count = 5
Task 1 [Heap]: Box=999, Hist=[2, 3, 4, 5, 6]
QEMU: Terminated
```
