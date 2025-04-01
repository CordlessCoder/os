use spinlock::SpinLock;
use x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError,
    },
};

use super::{
    // bump_alloc::{BumpAlloc, SpinLockBump},
    freelist_alloc::{FreeListAlloc, SpinLockFreelist},
};

// #[global_allocator]
// static ALLOCATOR: SpinLockBump = SpinLockBump(SpinLock::disable_interrupts(BumpAlloc::empty()));
#[global_allocator]
pub static ALLOCATOR: SpinLockFreelist =
    SpinLockFreelist(SpinLock::disable_interrupts(FreeListAlloc::empty()));

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 256 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_alloc: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1;
        let heap_start = Page::containing_address(heap_start);
        let heap_end = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start, heap_end)
    };

    for page in page_range {
        let frame = frame_alloc
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_alloc)?.flush() };
    }

    // unsafe {
    //     *ALLOCATOR.0.lock() = BumpAlloc::init(NonZeroUsize::new(HEAP_START).unwrap(), HEAP_SIZE);
    // }
    unsafe {
        ALLOCATOR.0.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
