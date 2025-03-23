use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
};

pub struct BootInfoFrameAllocator {
    memory: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// # Safety
    /// The passed memory map must be valid. The main requirement is
    /// that all frames that are marked as `USABLE` in it are really unused.
    pub unsafe fn new(memory: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory, next: 0 }
    }
}
// TODO: Write impl for FrameAllocator
impl BootInfoFrameAllocator {
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory.iter();
        let usable = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addrs = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addrs.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
