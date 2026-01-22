// === FILE: ./eos1/src/syscall.rs ===
use crate::task::{self, Task, TaskState};
use crate::mm::page_table::{new_user_page_table, PTE_U, PTE_R, PTE_W, translate};
use crate::mm::{frame, page_table};
use crate::fs;
use crate::elf;
use crate::plic;
use alloc::vec::Vec;

pub const PUTCHAR: u64 = 1;
pub const GETCHAR: u64 = 2;
pub const FILE_LEN: u64 = 3;
pub const FILE_READ: u64 = 4;
pub const FILE_LIST: u64 = 5;
pub const FILE_WRITE: u64 = 8; 
pub const CHDIR: u64 = 9;
pub const EXEC: u64 = 6;
pub const DISK_READ: u64 = 7;
pub const EXIT: u64 = 93;
pub const SCHED_YIELD: u64 = 124;
pub const GETPID: u64 = 172;
pub const WAIT: u64 = 260; 

unsafe fn user_to_kernel_ptr<T>(vaddr: usize, current_task: &Task) -> Option<*mut T> {
    if vaddr >= 0x8000_0000 && vaddr < 0x8800_0000 { return Some(vaddr as *mut T); }
    let root_ptr = (current_task.root_ppn << 12) as *const page_table::PageTable;
    if root_ptr.is_null() { return None; }
    let root = unsafe { &*root_ptr };
    if let Some(paddr) = unsafe { translate(root, vaddr) } {
        let offset = vaddr & 0xFFF;
        Some((paddr + offset) as *mut T)
    } else { None }
}

pub unsafe fn dispatcher(ctx: &mut crate::task::Context) -> *mut crate::task::Context {
    let id = ctx.regs[17];
    let a0 = ctx.regs[10];
    let a1 = ctx.regs[11];
    let a2 = ctx.regs[12];
    let a3 = ctx.regs[13];

    let scheduler = task::get_scheduler();

    match id {
        PUTCHAR => print!("{}", a0 as u8 as char),
        GETCHAR => ctx.regs[10] = plic::pop_key().unwrap_or(0) as u64,
        
        SCHED_YIELD => {
            // [關鍵修正] 
            // 因為我們要切換 Context，不會執行函式底部的 `ctx.mepc += 4`
            // 所以必須在這裡手動推進 PC，否則下次醒來會再次執行 ecall (無限 Yield)
            ctx.mepc += 4; 
            return unsafe { scheduler.schedule() };
        },

        GETPID => {
            ctx.regs[10] = scheduler.current_task().id as u64;
        },

        EXIT => {
            let exit_code = a0 as i32;
            let current = scheduler.current_task();
            current.state = TaskState::Zombie;
            current.exit_code = exit_code;
            // EXIT 不需要 +=4，因為這個 Task 不會再醒來了
            return unsafe { scheduler.schedule() };
        },

        WAIT => {
            let _pid = a0 as isize; 
            let code_ptr_vaddr = a1 as usize; 

            let mut zombie_idx = None;
            let mut has_children = false;

            for (i, t) in scheduler.tasks.iter().enumerate() {
                if t.id > 1 { // 忽略 Shell 和 Idle
                    has_children = true;
                    if t.state == TaskState::Zombie {
                        zombie_idx = Some(i);
                        break;
                    }
                }
            }

            if let Some(idx) = zombie_idx {
                let t = scheduler.tasks.remove(idx);
                // 修正索引位移
                if scheduler.current_index >= idx && scheduler.current_index > 0 {
                    scheduler.current_index -= 1;
                }

                if let Some(kptr) = unsafe { user_to_kernel_ptr::<i32>(code_ptr_vaddr, scheduler.current_task()) } {
                     unsafe { *kptr = t.exit_code; }
                }

                ctx.regs[10] = t.id as u64;
            } else if has_children {
                ctx.regs[10] = (-1isize) as u64;
            } else {
                ctx.regs[10] = (-2isize) as u64;
            }
        },

        FILE_LEN => {
            let current_task = scheduler.current_task();
            if let Some(kptr) = unsafe { user_to_kernel_ptr::<u8>(a0 as usize, current_task) } {
                let slice = unsafe { core::slice::from_raw_parts(kptr, a1 as usize) };
                let fname = core::str::from_utf8(slice).unwrap_or("");
                if let Some(data) = fs::get_file_content(fname) { ctx.regs[10] = data.len() as u64; }
                else { ctx.regs[10] = (-1isize) as u64; }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        FILE_READ => {
            let current_task = scheduler.current_task();
            let ptrs = unsafe { (user_to_kernel_ptr::<u8>(a0 as usize, current_task), user_to_kernel_ptr::<u8>(a2 as usize, current_task)) };
            if let (Some(kname), Some(kbuf)) = ptrs {
                unsafe {
                    let fname = core::str::from_utf8(core::slice::from_raw_parts(kname, a1 as usize)).unwrap_or("");
                    let user_buf = core::slice::from_raw_parts_mut(kbuf, a3 as usize);
                    if let Some(data) = fs::get_file_content(fname) {
                        let len = core::cmp::min(data.len(), user_buf.len());
                        user_buf[..len].copy_from_slice(&data[..len]);
                        ctx.regs[10] = len as u64;
                    } else { ctx.regs[10] = (-1isize) as u64; }
                }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        FILE_WRITE => {
            let current_task = scheduler.current_task();
            let ptrs = unsafe { (user_to_kernel_ptr::<u8>(a0 as usize, current_task), user_to_kernel_ptr::<u8>(a2 as usize, current_task)) };
            if let (Some(kname), Some(kdata)) = ptrs {
                unsafe {
                    let fname = core::str::from_utf8(core::slice::from_raw_parts(kname, a1 as usize)).unwrap_or("");
                    let data = core::slice::from_raw_parts(kdata, a3 as usize);
                    ctx.regs[10] = fs::write_file(fname, data) as u64;
                }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        CHDIR => {
            let current_task = scheduler.current_task();
            if let Some(kptr) = unsafe { user_to_kernel_ptr::<u8>(a0 as usize, current_task) } {
                unsafe {
                    let fname = core::str::from_utf8(core::slice::from_raw_parts(kptr, a1 as usize)).unwrap_or("");
                    ctx.regs[10] = fs::change_dir(fname) as u64;
                }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        FILE_LIST => {
            let current_task = scheduler.current_task();
            if let Some(kptr) = unsafe { user_to_kernel_ptr::<u8>(a1 as usize, current_task) } {
                unsafe {
                    let user_buf = core::slice::from_raw_parts_mut(kptr, a2 as usize);
                    let files = fs::list_files();
                    if (a0 as usize) < files.len() {
                        let (ftype, name) = &files[a0 as usize];
                        let display_name = if *ftype == 1 { alloc::format!("{}/", name) } else { alloc::format!("{}", name) };
                        let bytes = display_name.as_bytes();
                        let len = core::cmp::min(bytes.len(), user_buf.len());
                        user_buf[..len].copy_from_slice(&bytes[..len]);
                        ctx.regs[10] = len as u64;
                    } else { ctx.regs[10] = (-1isize) as u64; }
                }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        EXEC => {
            let current_task = scheduler.current_task();
            let ptrs = unsafe { (user_to_kernel_ptr::<u8>(a0 as usize, current_task), user_to_kernel_ptr::<&str>(a2 as usize, current_task)) };
            if let (Some(kelf), Some(kargv)) = ptrs {
                unsafe {
                    let elf_data = core::slice::from_raw_parts(kelf, a1 as usize);
                    let argv_ptr = kargv; 
                    let argc = a3 as usize;
                    let argv_slice = core::slice::from_raw_parts(argv_ptr, argc);
                    
                    let new_table = new_user_page_table();
                    if new_table.is_null() { ctx.regs[10] = (-1isize) as u64; }
                    else {
                        if let Some(entry) = elf::load_elf(elf_data, &mut *new_table) {
                            let stack_frame = frame::alloc_frame();
                            let stack_vaddr = 0xF000_0000;
                            page_table::map(&mut *new_table, stack_vaddr, stack_frame, PTE_U | PTE_R | PTE_W);
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
                            for (i, vaddr) in str_vaddrs.iter().enumerate() { *ptr_array.add(i) = *vaddr; }
                            *ptr_array.add(str_vaddrs.len()) = 0; 
                            let sp_vaddr = stack_vaddr + (sp_paddr - stack_frame);
                            let new_pid = scheduler.tasks.len();
                            let mut new_task = Task::new_user(new_pid);
                            new_task.root_ppn = (new_table as usize) >> 12;
                            new_task.context.mepc = entry;
                            new_task.context.regs[2] = sp_vaddr as u64;
                            new_task.context.regs[10] = argc as u64;
                            new_task.context.regs[11] = argv_vaddr as u64;
                            scheduler.spawn(new_task);
                            ctx.regs[10] = new_pid as u64;
                        } else { ctx.regs[10] = (-1isize) as u64; }
                    }
                }
            } else { ctx.regs[10] = (-1isize) as u64; }
        },
        DISK_READ => {
            let sector = a0;
            let current_task = scheduler.current_task();
            if let Some(kbuf) = unsafe { user_to_kernel_ptr::<u8>(a1 as usize, current_task) } {
                let data = crate::virtio::read_disk(sector);
                unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), kbuf, 512); }
            }
        },
        _ => println!("Unknown Syscall: {}", id),
    }
    
    // 只有非排程相關的 Syscall 才會執行到這裡
    ctx.mepc += 4;
    ctx
}