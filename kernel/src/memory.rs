pub mod frame_alloc;
use core::sync::atomic::AtomicBool;
use x86_64::{
    VirtAddr,
    structures::paging::{PageTable, mapper::OffsetPageTable},
};

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
/// The caller must guarantee that the complete physical memory is
/// mapped to virtual memory at the passed `physical_memory_offset`.
/// Also, this function must be only called once to avoid aliasing
/// `&mut` references (which is undefined behavior).
pub unsafe fn get_table(phys_offset: VirtAddr) -> OffsetPageTable<'static> {
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
