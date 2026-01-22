// === FILE: ./eos1/src/main.rs ===
#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

#[macro_use] mod uart;
mod task;
mod heap;
mod fs;
mod elf;
mod mm;
mod plic;
mod timer;
mod trap;
mod syscall;
mod virtio;
mod shell; 

use core::panic::PanicInfo;
use task::{Task, Scheduler};
#[allow(unused_imports)]
use crate::mm::page_table::{PageTable, PTE_R, PTE_W, PTE_X, PTE_U, KERNEL_PAGE_TABLE};

core::arch::global_asm!(include_str!("entry.S"));
core::arch::global_asm!(include_str!("trap.S"));

unsafe extern "C" { fn trap_vector(); }

#[unsafe(no_mangle)]
pub extern "C" fn rust_main() -> ! {
    println!("-----------------------------------");
    println!("   EOS Refactored (v1.0)           ");
    println!("-----------------------------------");

    unsafe {
        core::arch::asm!("csrw pmpaddr0, {}", in(reg) !0usize);
        core::arch::asm!("csrw pmpcfg0, {}", in(reg) 0x1Fusize);

        mm::frame::init();
        heap::init();
        
        let root_ptr = mm::frame::alloc_frame() as *mut PageTable;
        let root = &mut *root_ptr;
        mm::page_table::KERNEL_PAGE_TABLE = root_ptr;

        mm::page_table::map(root, 0x1000_0000, 0x1000_0000, PTE_R | PTE_W); 
        
        let mut addr = 0x0200_0000;
        while addr < 0x0200_FFFF { mm::page_table::map(root, addr, addr, PTE_R | PTE_W); addr += 4096; } 
        
        println!("[Kernel] Mapping MMIO (PLIC & VirtIO)...");
        let mut addr = 0x0C00_0000;
        let end_plic = 0x0C20_1000; 
        while addr < end_plic { mm::page_table::map(root, addr, addr, PTE_R | PTE_W); addr += 4096; } 
        
        let mut addr = 0x1000_0000;
        let end_mmio = 0x1000_8000;
        while addr < end_mmio { mm::page_table::map(root, addr, addr, PTE_R | PTE_W); addr += 4096; }

        let start = 0x8000_0000; let end = 0x8800_0000; 
        let mut addr = start;
        while addr < end { mm::page_table::map(root, addr, addr, PTE_R | PTE_W | PTE_X | PTE_U); addr += 4096; }

        let satp_val = (8 << 60) | ((root_ptr as usize) >> 12);
        core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) satp_val);
        println!("[Kernel] MMU Enabled.");

        Scheduler::init();
        let scheduler = task::get_scheduler();
        scheduler.spawn(Task::new_kernel(0, shell::shell_entry));
        scheduler.spawn(Task::new_kernel(1, shell::bg_task));

        plic::init();
        virtio::init();
        println!("[Kernel] Devices Initialized.");

        // [關鍵修正] 使用 Direct Mode (移除 | 1)
        // 這樣所有的 Trap 都會正確跳轉到 trap_vector 入口
        core::arch::asm!("csrw mtvec, {}", in(reg) (trap_vector as usize));
        
        let first_task = &mut scheduler.tasks[0];
        core::arch::asm!("csrw mscratch, {}", in(reg) &mut first_task.context);
        
        let mstatus: usize = (0 << 11) | (1 << 7) | (1 << 13);
        core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
        
        timer::set_next();
        core::arch::asm!("csrrs zero, mie, {}", in(reg) (1 << 11) | (1 << 7));

        println!("[OS] System Ready. Switching to Shell...");
        
        core::arch::asm!(
            "mv sp, {}",
            "csrw mepc, {}",
            "mret",
            in(reg) first_task.context.regs[2],
            in(reg) first_task.context.mepc
        );
    }
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! { println!("\n[PANIC] {}", info); loop {} }