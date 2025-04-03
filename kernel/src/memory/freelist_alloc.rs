use core::{
    alloc::{GlobalAlloc, Layout},
    mem,
    ptr::{self, NonNull},
};
use spinlock::{DisableInterrupts, SpinLock};
/// A fairly simple FreeList-backed heap allocator.
pub struct FreeListAlloc {
    total: usize,
    head: ListNode,
}

struct ListNode {
    size: usize,
    next: Option<NonNull<ListNode>>,
}
unsafe impl Sync for ListNode {}
unsafe impl Send for ListNode {}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }
    fn start_addr(&self) -> usize {
        self as *const _ as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

impl FreeListAlloc {
    /// Create a FreeListAlloc with no backing memory.
    pub const fn empty() -> Self {
        FreeListAlloc {
            head: ListNode::new(0),
            total: 0,
        }
    }
    /// # Safety
    /// Must be called with an address range that the allocator can freely create mutable
    /// references into
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        unsafe {
            self.add_free_region(start, size);
        }
        self.total = size;
    }
    unsafe fn add_free_region(&mut self, addr: usize, mut size: usize) {
        assert_eq!(
            addr.next_multiple_of(mem::align_of::<ListNode>()),
            addr,
            "The free region must be well-aligned to hold a ListNode"
        );
        assert!(
            size >= mem::size_of::<ListNode>(),
            "The free region must be large enough to hold a ListNode"
        );

        let mut closest_before = &mut self.head;
        while let Some(mut node) = closest_before.next {
            if node.addr().get() >= addr {
                break;
            }
            closest_before = unsafe { node.as_mut() };
        }
        let mut merge_left = false;
        if closest_before.end_addr() == addr {
            merge_left = true;
        }
        if let Some(mut next) = closest_before.next {
            let next = unsafe { next.as_mut() };
            if next.start_addr() == addr + size {
                size += next.size;
                let next = next.next.take();
                closest_before.next = next;
            }
        }
        if merge_left {
            closest_before.size += size;
            return;
        }
        let mut node = ListNode::new(size);
        node.next = closest_before.next.take();
        let node_ptr = addr as *mut ListNode;
        let node_ptr =
            NonNull::new(node_ptr).expect("add_free_region cannot be called with a null pointer");
        unsafe {
            node_ptr.write(node);
            closest_before.next = Some(node_ptr);
        }
    }
    pub fn set_total(&mut self, free: usize) {
        self.total = free;
    }
    unsafe fn alloc(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let mut cur = &mut self.head as *mut ListNode;
        unsafe {
            while let Some(mut region) = (*cur).next {
                let Ok(RegionAllocSplit {
                    start_node,
                    addr,
                    end_node,
                }) = Self::alloc_in_region(region.as_ref(), size, align)
                else {
                    cur = region.as_ptr();
                    continue;
                };
                // Remove region
                let next = region.as_mut().next.take();
                (*cur).next = next;
                if let Some((addr, len)) = start_node {
                    self.add_free_region(addr, len);
                }
                if let Some((addr, len)) = end_node {
                    self.add_free_region(addr, len);
                }
                return Some(addr as *mut u8);
            }
        };
        None
    }
    fn alloc_in_region(
        region: &ListNode,
        size: usize,
        align: usize,
    ) -> Result<RegionAllocSplit, ()> {
        const MIN: usize = mem::size_of::<ListNode>();
        let mut start_node = None;
        let mut end_node = None;
        let mut start_addr = region.start_addr().next_multiple_of(align);
        if start_addr != region.start_addr() {
            // Requires padding at the start for alignment
            start_addr = (region.start_addr() + mem::size_of::<ListNode>()).next_multiple_of(align);
            let start_size = start_addr - region.start_addr();
            if start_size < MIN {
                return Err(());
            }
            start_node = Some((region.start_addr(), start_size));
        }
        let end_addr = start_addr + size;
        if end_addr < region.end_addr() {
            // Requires padding at the end for alignment
            let end_size = region.end_addr() - end_addr;
            if end_size < MIN {
                return Err(());
            }
            end_node = Some((end_addr, end_size));
        }
        if end_addr > region.end_addr() {
            return Err(());
        }
        Ok(RegionAllocSplit {
            start_node,
            addr: start_addr,
            end_node,
        })
    }
    fn prepare_layout(mut layout: Layout) -> Layout {
        layout = layout.align_to(mem::size_of::<ListNode>()).unwrap();
        layout.pad_to_align()
    }
    pub fn stats(&self) -> AllocStats {
        let mut regions = 0;
        let mut free_mem = 0;
        let mut cur = &self.head;
        while let Some(node) = cur.next {
            regions += 1;
            cur = unsafe { node.as_ref() };
            free_mem += cur.size;
        }
        AllocStats {
            free_regions: regions,
            free_memory: free_mem,
            total: self.total,
            used: self.total.wrapping_sub(free_mem),
        }
    }
}
#[derive(Debug)]
pub struct AllocStats {
    pub free_regions: usize,
    pub free_memory: usize,
    pub total: usize,
    pub used: usize,
}
struct RegionAllocSplit {
    /// The address and length of the start free node
    start_node: Option<(usize, usize)>,
    /// The address of the allocation
    addr: usize,
    /// The address and length of the end free node
    end_node: Option<(usize, usize)>,
}

pub struct SpinLockFreelist(pub SpinLock<FreeListAlloc, DisableInterrupts>);

unsafe impl GlobalAlloc for SpinLockFreelist {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let layout = FreeListAlloc::prepare_layout(layout);
        unsafe {
            let Some(ptr) = self.0.lock().alloc(layout.size(), layout.align()) else {
                return ptr::null_mut();
            };
            ptr
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let layout = FreeListAlloc::prepare_layout(layout);
        unsafe {
            self.0.lock().add_free_region(ptr as usize, layout.size());
        }
    }
}
