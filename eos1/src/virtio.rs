use crate::mm::frame::alloc_frame;
use core::mem::size_of;

// --- VirtIO MMIO 暫存器偏移量 ---
const VIRTIO0: usize = 0x1000_1000;
const MAGIC: usize = 0x000;
const VERSION: usize = 0x004;
const DEVICE_ID: usize = 0x008;
const DEVICE_FEATURES: usize = 0x010;
const DRIVER_FEATURES: usize = 0x020;
const GUEST_PAGE_SIZE: usize = 0x028;
const QUEUE_SEL: usize = 0x030;
const QUEUE_NUM_MAX: usize = 0x034;
const QUEUE_NUM: usize = 0x038;
const QUEUE_PFN: usize = 0x040;
const QUEUE_NOTIFY: usize = 0x050;
const STATUS: usize = 0x070;

// --- VirtIO 狀態位元 ---
const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_DRIVER_OK: u32 = 4;

// --- Virtqueue 結構 ---
#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; 32],
    #[allow(dead_code)]
    used_event: u16,
}

#[repr(C)]
struct VirtqUsedElem {
    #[allow(dead_code)]
    id: u32,
    #[allow(dead_code)]
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    #[allow(dead_code)]
    flags: u16,
    idx: u16,
    #[allow(dead_code)]
    ring: [VirtqUsedElem; 32],
    #[allow(dead_code)]
    avail_event: u16,
}

// --- Driver 狀態 ---
static mut QUEUE_PAGE: usize = 0;
static mut USED_IDX: u16 = 0;

// Flags & Types
const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;
const VIRTIO_BLK_T_IN: u32 = 0;  // Read

#[repr(C)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
}

/// 初始化 VirtIO 驅動
pub fn init() {
    unsafe {
        let base = VIRTIO0 as *mut u32;
        
        if base.add(MAGIC / 4).read_volatile() != 0x74726976 { panic!("VirtIO magic mismatch"); }
        if base.add(VERSION / 4).read_volatile() != 1 { panic!("VirtIO version mismatch"); }
        if base.add(DEVICE_ID / 4).read_volatile() != 2 { panic!("Not a block device"); }

        base.add(STATUS / 4).write_volatile(0); // Reset

        let mut status = STATUS_ACKNOWLEDGE | STATUS_DRIVER;
        base.add(STATUS / 4).write_volatile(status);

        let _features = base.add(DEVICE_FEATURES / 4).read_volatile();
        base.add(DRIVER_FEATURES / 4).write_volatile(0);
        
        status |= STATUS_FEATURES_OK;
        base.add(STATUS / 4).write_volatile(status);

        base.add(QUEUE_SEL / 4).write_volatile(0);
        
        let max = base.add(QUEUE_NUM_MAX / 4).read_volatile();
        if max == 0 { panic!("VirtIO queue max is 0"); }

        base.add(QUEUE_NUM / 4).write_volatile(32);

        // 分配兩頁以確保空間足夠
        let page1 = alloc_frame();
        let _page2 = alloc_frame(); 
        if page1 == 0 { panic!("VirtIO OOM"); }
        QUEUE_PAGE = page1;
        
        base.add(GUEST_PAGE_SIZE / 4).write_volatile(4096);
        base.add(QUEUE_PFN / 4).write_volatile((page1 >> 12) as u32);

        status |= STATUS_DRIVER_OK;
        base.add(STATUS / 4).write_volatile(status);
    }
}

/// 讀取磁碟的一個 Sector (512 bytes)
pub fn read_disk(sector: u64) -> [u8; 512] {
    let buffer = [0u8; 512];
    
    unsafe {
        let base = VIRTIO0 as *mut u32;
        let desc_table = QUEUE_PAGE as *mut VirtqDesc;
        let avail_ring = (QUEUE_PAGE + 512) as *mut VirtqAvail;
        let used_ring = (QUEUE_PAGE + 4096) as *mut VirtqUsed;

        // 1. Request Header
        static mut REQ: VirtioBlkReq = VirtioBlkReq {
            type_: VIRTIO_BLK_T_IN,
            reserved: 0,
            sector: 0,
        };
        REQ.sector = sector;

        // 2. Status Byte
        static mut STATUS_BYTE: u8 = 255;
        STATUS_BYTE = 255;

        // 3. Fill Descriptors
        // Desc 0: Header
        (*desc_table.add(0)).addr = &raw mut REQ as u64;
        (*desc_table.add(0)).len = size_of::<VirtioBlkReq>() as u32;
        (*desc_table.add(0)).flags = VRING_DESC_F_NEXT;
        (*desc_table.add(0)).next = 1;

        // Desc 1: Buffer (Write-only for device)
        (*desc_table.add(1)).addr = buffer.as_ptr() as u64;
        (*desc_table.add(1)).len = 512;
        (*desc_table.add(1)).flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
        (*desc_table.add(1)).next = 2;

        // Desc 2: Status (Write-only for device)
        (*desc_table.add(2)).addr = &raw mut STATUS_BYTE as u64;
        (*desc_table.add(2)).len = 1;
        (*desc_table.add(2)).flags = VRING_DESC_F_WRITE;
        (*desc_table.add(2)).next = 0;

        // 4. Update Available Ring
        let idx = (*avail_ring).idx as usize;
        (*avail_ring).ring[idx % 32] = 0; // Head Index
        
        core::arch::asm!("fence");
        (*avail_ring).idx = (*avail_ring).idx.wrapping_add(1);

        // 5. Notify Device
        base.add(QUEUE_NOTIFY / 4).write_volatile(0);

        // 6. Wait for completion (Spinning)
        while (*used_ring).idx == USED_IDX {
            core::arch::asm!("nop");
        }
        USED_IDX = (*used_ring).idx;
    }

    buffer
}