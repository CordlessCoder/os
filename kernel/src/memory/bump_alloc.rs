use core::{
    alloc::{GlobalAlloc, Layout},
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

use spinlock::{DisableInterrupts, SpinLock};
/// A simple bump allocator implementation.
pub struct BumpAlloc {
    start_addr: NonZeroUsize,
    len: usize,
    cursor: usize,
    alloc_count: usize,
}

impl BumpAlloc {
    /// Create a BumpAlloc with no backing memory.
    pub const fn empty() -> Self {
        BumpAlloc {
            start_addr: NonZeroUsize::new(1).unwrap(),
            len: 0,
            cursor: 0,
            alloc_count: 0,
        }
    }
    /// # Safety
    /// Must be called with an address range that the allocator can freely create mutable
    /// references into
    pub unsafe fn init(start_addr: NonZeroUsize, len: usize) -> Self {
        BumpAlloc {
            start_addr,
            len,
            ..Self::empty()
        }
    }
    pub fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        if layout.size() == 0 {
            return Some(NonNull::dangling());
        }
        let aligned_cursor = self.cursor.checked_next_multiple_of(layout.align())?;
        let end = aligned_cursor.checked_add(layout.size())?;
        if end > self.len {
            return None;
        }
        self.cursor = end;
        self.alloc_count += 1;
        NonNull::new((self.start_addr.get() + aligned_cursor) as *mut u8)
    }
    pub fn dealloc(&mut self) {
        self.alloc_count -= 1;
        if self.alloc_count == 0 {
            self.cursor = 0
        }
    }
}

pub struct SpinLockBump(pub SpinLock<BumpAlloc, DisableInterrupts>);

unsafe impl GlobalAlloc for SpinLockBump {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let Some(ptr) = self.0.lock().alloc(layout) else {
            return ptr::null_mut();
        };
        ptr.as_ptr()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        self.0.lock().dealloc();
    }
}
