pub mod bump_alloc;
pub mod frame_alloc;
pub mod freelist_alloc;
pub mod global_alloc;
use bootloader::BootInfo;
use core::sync::atomic::AtomicBool;
use x86_64::{
    VirtAddr,
    structures::paging::{PageTable, mapper::OffsetPageTable},
};

/// Initialize the ALLOCATOR.
pub fn init(boot_info: &'static BootInfo) {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { get_table(phys_offset) };
    let mut frame_alloc =
        unsafe { frame_alloc::BootInfoFrameAllocator::new(&boot_info.memory_map) };
    global_alloc::init_heap(&mut mapper, &mut frame_alloc).unwrap();
}

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// The caller must guarantee that the complete physical memory is
/// mapped to virtual memory at the passed `physical_memory_offset`.
/// Also, this function must be only called once to avoid aliasing
/// `&mut` references (which is undefined behavior).
unsafe fn get_table(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
    // Guard against accidental double calls
    static CALLED: AtomicBool = AtomicBool::new(false);
    assert!(
        !CALLED.swap(true, core::sync::atomic::Ordering::AcqRel),
        "get_table called more than once"
    );
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let page_table: *mut PageTable = virt.as_mut_ptr();

    unsafe { OffsetPageTable::new(&mut *page_table, phys_offset) }
}
