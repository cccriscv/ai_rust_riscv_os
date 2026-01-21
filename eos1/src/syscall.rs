use crate::task::{self, Task};
use crate::mm::page_table::{new_user_page_table, PTE_U, PTE_R, PTE_W, KERNEL_PAGE_TABLE};
use crate::mm::{frame, page_table};
use crate::fs;
use crate::elf;
use crate::plic;
// [修正] 移除未使用的 timer 引用
use alloc::vec::Vec;

// Syscall Constants
pub const PUTCHAR: u64 = 1;
pub const GETCHAR: u64 = 2;
pub const FILE_LEN: u64 = 3;
pub const FILE_READ: u64 = 4;
pub const FILE_LIST: u64 = 5;
pub const EXEC: u64 = 6;
pub const DISK_READ: u64 = 7;
pub const EXIT: u64 = 93;

pub unsafe fn dispatcher(ctx: &mut crate::task::Context) -> *mut crate::task::Context {
    let id = ctx.regs[17];
    let a0 = ctx.regs[10];
    let a1 = ctx.regs[11];
    let a2 = ctx.regs[12];
    let a3 = ctx.regs[13];

    match id {
        PUTCHAR => print!("{}", a0 as u8 as char),
        GETCHAR => ctx.regs[10] = plic::pop_key().unwrap_or(0) as u64,
        
        FILE_LEN => {
            // [修正] 包裹 unsafe
            let slice = unsafe { core::slice::from_raw_parts(a0 as *const u8, a1 as usize) };
            let fname = core::str::from_utf8(slice).unwrap_or("");
            if let Some(data) = fs::get_file_content(fname) { ctx.regs[10] = data.len() as u64; }
            else { ctx.regs[10] = (-1isize) as u64; }
        },
        FILE_READ => {
            // [修正] 包裹 unsafe
            unsafe {
                let slice = core::slice::from_raw_parts(a0 as *const u8, a1 as usize);
                let fname = core::str::from_utf8(slice).unwrap_or("");
                let user_buf = core::slice::from_raw_parts_mut(a2 as *mut u8, a3 as usize);
                
                if let Some(data) = fs::get_file_content(fname) {
                    let len = core::cmp::min(data.len(), user_buf.len());
                    user_buf[..len].copy_from_slice(&data[..len]);
                    ctx.regs[10] = len as u64;
                } else { ctx.regs[10] = (-1isize) as u64; }
            }
        },
        FILE_LIST => {
            // [修正] 包裹 unsafe
            unsafe {
                let user_buf = core::slice::from_raw_parts_mut(a1 as *mut u8, a2 as usize);
                let files = fs::list_files();
                if (a0 as usize) < files.len() {
                    let fname = files[a0 as usize].as_bytes();
                    let len = core::cmp::min(fname.len(), user_buf.len());
                    user_buf[..len].copy_from_slice(&fname[..len]);
                    ctx.regs[10] = len as u64;
                } else { ctx.regs[10] = (-1isize) as u64; }
            }
        },
        
        EXEC => {
            // [修正] 包裹 unsafe
            unsafe {
                let elf_data = core::slice::from_raw_parts(a0 as *const u8, a1 as usize);
                let argv_ptr = a2 as *const &str;
                let argc = a3 as usize;
                let argv_slice = core::slice::from_raw_parts(argv_ptr, argc);

                println!("[Kernel] Spawning process with {} args...", argc);

                let new_table = new_user_page_table();
                if new_table.is_null() { ctx.regs[10] = (-1isize) as u64; }
                else {
                    if let Some(entry) = elf::load_elf(elf_data, &mut *new_table) {
                        println!("[Kernel] ELF loaded.");
                        
                        let stack_frame = frame::alloc_frame();
                        let stack_vaddr = 0xF000_0000;
                        page_table::map(&mut *new_table, stack_vaddr, stack_frame, PTE_U | PTE_R | PTE_W);

                        // Push Args logic
                        let stack_top_paddr = stack_frame + 4096;
                        let mut sp_paddr = stack_top_paddr;
                        let mut str_vaddrs = Vec::new();
                        
                        for arg in argv_slice {
                            let bytes = arg.as_bytes();
                            let len = bytes.len() + 1; 
                            sp_paddr -= len;
                            let dest = sp_paddr as *mut u8;
                            core::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
                            *dest.add(bytes.len()) = 0; 
                            str_vaddrs.push(stack_vaddr + (sp_paddr - stack_frame));
                        }
                        sp_paddr -= sp_paddr % 8;
                        sp_paddr -= (str_vaddrs.len() + 1) * 8; 
                        let argv_vaddr = stack_vaddr + (sp_paddr - stack_frame);
                        let ptr_array = sp_paddr as *mut usize;
                        for (i, vaddr) in str_vaddrs.iter().enumerate() {
                            *ptr_array.add(i) = *vaddr;
                        }
                        *ptr_array.add(str_vaddrs.len()) = 0; 
                        let sp_vaddr = stack_vaddr + (sp_paddr - stack_frame);

                        let scheduler = task::get_scheduler();
                        let new_pid = scheduler.tasks.len();
                        let mut new_task = Task::new_user(new_pid);
                        new_task.root_ppn = (new_table as usize) >> 12;
                        new_task.context.mepc = entry;
                        new_task.context.regs[2] = sp_vaddr as u64;
                        new_task.context.regs[10] = argc as u64;
                        new_task.context.regs[11] = argv_vaddr as u64;

                        scheduler.spawn(new_task);
                        println!("[Kernel] Process spawned with PID {}", new_pid);
                        ctx.regs[10] = new_pid as u64;
                    } else { ctx.regs[10] = (-1isize) as u64; }
                }
            }
        },
        
        DISK_READ => {
            let sector = a0;
            let buf_ptr = a1 as *mut u8;
            let data = crate::virtio::read_disk(sector);
            // [修正] 包裹 unsafe
            unsafe {
                core::ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr, 512);
            }
        },

        EXIT => {
            println!("[Kernel] Process exited code: {}", a0);
            let scheduler = task::get_scheduler();
            if scheduler.tasks.len() > 2 { scheduler.tasks.truncate(2); }
            
            // Switch back to Shell
            scheduler.current_index = 0;
            let shell_task = &mut scheduler.tasks[0];
            // [修正] 包裹 unsafe
            unsafe {
                let kernel_root = KERNEL_PAGE_TABLE as usize;
                core::arch::asm!("csrw satp, {}", "sfence.vma", in(reg) (8 << 60) | (kernel_root >> 12));
            }
            return &mut shell_task.context;
        },
        _ => println!("Unknown Syscall: {}", id),
    }
    
    ctx.mepc += 4;
    ctx
}