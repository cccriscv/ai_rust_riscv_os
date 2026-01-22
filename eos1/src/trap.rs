// === FILE: ./eos1/src/trap.rs ===
use crate::task::{self, Context};
use crate::syscall;
use crate::timer;
use crate::plic;
use crate::mm::page_table::KERNEL_PAGE_TABLE;
use crate::shell;

// 整合後的 Trap Handler
#[unsafe(no_mangle)]
pub extern "C" fn handle_trap(ctx_ptr: *mut Context) -> *mut Context {
    let mcause: usize;
    unsafe { core::arch::asm!("csrr {}, mcause", out(reg) mcause); }
    
    let is_interrupt = (mcause >> 63) != 0;
    let code = mcause & 0xfff;

    if is_interrupt {
        match code {
            7 => { // Machine Timer Interrupt
                timer::set_next();
                let scheduler = task::get_scheduler();
                return unsafe { scheduler.schedule() };
            }
            11 => { // Machine External Interrupt (PLIC)
                 plic::handle_interrupt();
                 return ctx_ptr;
            }
            _ => {
                println!("[Kernel] Unexpected interrupt: {}", code);
                return ctx_ptr;
            }
        }
    } else {
        if code == 8 { 
            return unsafe { syscall::dispatcher(&mut *ctx_ptr) };
        }
        
        let mtval: usize;
        unsafe { core::arch::asm!("csrr {}, mtval", out(reg) mtval); }
        println!("\n[Crash] mcause={}, mepc={:x}, mtval={:x}", code, unsafe { (*ctx_ptr).mepc }, mtval);
        println!("User App crashed. Rebooting shell...");
        
        unsafe {
            let kernel_root = KERNEL_PAGE_TABLE as usize;
            core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) (8 << 60) | (kernel_root >> 12));
            
            let scheduler = task::get_scheduler();
            if scheduler.tasks.len() > 2 { scheduler.tasks.truncate(2); }
            scheduler.current_index = 0;
            let shell_task = &mut scheduler.tasks[0];
            
            shell_task.root_ppn = 0;
            shell_task.context.mepc = shell::shell_entry as u64;
            
            let mut mstatus: usize;
            core::arch::asm!("csrr {}, mstatus", out(reg) mstatus);
            mstatus &= !(3 << 11); mstatus |= 1 << 7;
            core::arch::asm!("csrw mstatus, {}", in(reg) mstatus);
            
            return &mut shell_task.context;
        }
    }
}

// [關鍵新增] 滿足 Linker 的空殼函式
// 由於我們在 main.rs 使用 Direct Mode，這兩個函式實際上不會被硬體呼叫
// 但 trap.S 的向量表需要它們的符號存在
#[unsafe(no_mangle)]
pub extern "C" fn handle_timer(_ctx: *mut Context) -> *mut Context {
    panic!("handle_timer should not be called in Direct Mode");
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_external(_ctx: *mut Context) -> *mut Context {
    panic!("handle_external should not be called in Direct Mode");
}