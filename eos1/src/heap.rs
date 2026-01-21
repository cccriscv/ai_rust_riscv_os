use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::mem::{size_of, align_of};

// 1MB 堆積空間
const HEAP_SIZE: usize = 1024 * 1024;

// 使用 usize 對齊，確保 HEAP_MEMORY 起始位址至少是 8-byte 對齊
#[repr(align(16))]
struct HeapStorage([u8; HEAP_SIZE]);

static mut HEAP_MEMORY: HeapStorage = HeapStorage([0; HEAP_SIZE]);

struct ListNode {
    size: usize,
    next: *mut ListNode,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: null_mut() }
    }
}

pub struct LinkedListAllocator {
    head: *mut ListNode,
}

unsafe impl Sync for LinkedListAllocator {}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { head: null_mut() }
    }

    pub unsafe fn init(&mut self) {
        // [修正] 加上 unsafe 區塊
        let heap_start = unsafe { &raw mut HEAP_MEMORY.0 as usize };
        
        let align = align_of::<ListNode>();
        let start_aligned = (heap_start + align - 1) & !(align - 1);
        
        let heap_end = heap_start + HEAP_SIZE;
        let heap_size = heap_end - start_aligned;

        let ptr = start_aligned as *mut ListNode;
        unsafe {
            ptr.write(ListNode::new(heap_size));
        }
        
        self.head = ptr;
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let allocator = unsafe { &mut *(&raw mut ALLOCATOR) };
        
        let mut prev: *mut ListNode = null_mut();
        let mut curr = allocator.head;

        // 要求對齊
        let required_align = core::cmp::max(layout.align(), align_of::<ListNode>());
        // [關鍵修正] 要求最小大小：必須至少能塞下一個 ListNode，否則 dealloc 會溢出
        let min_size = size_of::<ListNode>();

        while !curr.is_null() {
            let (curr_addr, curr_size, curr_next) = unsafe {
                (curr as usize, (*curr).size, (*curr).next)
            };

            // 1. 計算起始位置
            let alloc_start = align_up(curr_addr, required_align);
            
            // 2. 計算使用者需要的結束位置
            let actual_req_size = core::cmp::max(layout.size(), min_size); // 取較大者
            
            let alloc_end = match alloc_start.checked_add(actual_req_size) {
                Some(end) => end,
                None => return null_mut(),
            };

            // 3. 下一個節點的起始位置也必須對齊
            let new_node_start = align_up(alloc_end, align_of::<ListNode>());

            let region_end = curr_addr + curr_size;

            if new_node_start <= region_end {
                let remaining_size = region_end - new_node_start;
                
                if remaining_size >= size_of::<ListNode>() {
                    let new_node_ptr = new_node_start as *mut ListNode;
                    unsafe {
                        (*new_node_ptr).size = remaining_size;
                        (*new_node_ptr).next = curr_next;
                    }

                    if prev.is_null() {
                        allocator.head = new_node_ptr;
                    } else {
                        unsafe { (*prev).next = new_node_ptr; }
                    }
                } else {
                    // 不切割，直接給整塊
                    if prev.is_null() {
                        allocator.head = curr_next;
                    } else {
                        unsafe { (*prev).next = curr_next; }
                    }
                }

                return alloc_start as *mut u8;
            }

            prev = curr;
            curr = curr_next;
        }

        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocator = unsafe { &mut *(&raw mut ALLOCATOR) };

        // 因為 alloc 時已經強制 min_size，所以這裡寫入是安全的
        let min_size = size_of::<ListNode>();
        // 注意：這裡計算 true_size 只是為了放進 ListNode.size，
        // 實際上我們可能回收了更多 (因為 alloc 時的 padding)，但我們無法得知確切大小
        // 這是簡易實作的缺陷 (會洩漏 padding)，但不影響穩定性
        let true_size = core::cmp::max(layout.size(), min_size);
        
        let new_node_ptr = ptr as *mut ListNode;
        
        unsafe {
            (*new_node_ptr).size = true_size;
            (*new_node_ptr).next = allocator.head;
        }
        
        allocator.head = new_node_ptr;
    }
}

#[global_allocator]
static mut ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

pub fn init() {
    unsafe {
        let ptr = &raw mut ALLOCATOR;
        (*ptr).init();
    }
}