use crate::task::{self, Context};
use crate::syscall;
use crate::timer;
use crate::plic;
use crate::mm::page_table::KERNEL_PAGE_TABLE;
use crate::shell; // 稍後我們會把 shell 移出去

#[unsafe(no_mangle)]
pub extern "C" fn handle_timer(_ctx_ptr: *mut Context) -> *mut Context {
    timer::set_next();
    let scheduler = task::get_scheduler();
    unsafe { scheduler.schedule() }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_external(ctx_ptr: *mut Context) -> *mut Context {
    plic::handle_interrupt();
    ctx_ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }
    
    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
        println!("[Kernel] Unexpected interrupt: {}", code);
        return ctx_ptr;
    } else {
        if code == 8 { 
            // 轉發給 syscall dispatcher
            return unsafe { syscall::dispatcher(&mut *ctx_ptr) };
        }
        
        // Crash Handling
        let mtval: usize;
        unsafe { core::arch::asm!("csrr {}, mtval", out(reg) mtval); }
        println!("\n[Crash] mcause={}, mepc={:x}, mtval={:x}", code, unsafe { (*ctx_ptr).mepc }, mtval);
        println!("User App crashed. Rebooting shell...");
        
        unsafe {
            // 重置為核心頁表
            let kernel_root = KERNEL_PAGE_TABLE as usize;
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) (8 << 60) | (kernel_root >> 12));
            
            // 清理任務
            let scheduler = task::get_scheduler();
            if scheduler.tasks.len() > 2 { scheduler.tasks.truncate(2); }
            scheduler.current_index = 0;
            let shell_task = &mut scheduler.tasks[0];
            
            shell_task.root_ppn = 0;
            // 指向 Shell 入口
            shell_task.context.mepc = shell::shell_entry as u64;
            
            // 重置 mstatus
            let mut mstatus: usize;
            core::arch::asm!("csrr {}, mstatus", out(reg) mstatus);
            mstatus &= !(3 << 11); mstatus |= 1 << 7;
            core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
            
            return &mut shell_task.context;
        }
    }
}