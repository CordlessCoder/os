use core::{
    alloc::{GlobalAlloc, Layout},
    mem,
    ptr::{self, NonNull},
};
use spinlock::{DisableInterrupts, SpinLock};
pub struct FreeListAlloc {
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
    pub const fn empty() -> Self {
        FreeListAlloc {
            head: ListNode::new(0),
        }
    }
    /// # Safety
    /// Must be called with an address range that the allocator can freely create mutable
    /// references into
    pub unsafe fn init(&mut self, start: usize, len: usize) {
        unsafe {
            self.add_free_region(start, len);
        }
    }
    unsafe fn add_free_region(&mut self, addr: usize, len: usize) {
        assert_eq!(
            addr.next_multiple_of(mem::align_of::<ListNode>()),
            addr,
            "The free region must be well-aligned to hold a ListNode"
        );
        assert!(
            len >= mem::size_of::<ListNode>(),
            "The free region must be large enough to hold a ListNode"
        );

        let mut node = ListNode::new(len);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        let node_ptr =
            NonNull::new(node_ptr).expect("add_free_region cannot be called with a null pointer");
        unsafe {
            node_ptr.write(node);
            self.head.next = Some(node_ptr);
        }
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
