use spinlock::LazyStatic;
use x86_64::VirtAddr;
use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    tss::TaskStateSegment,
};

static GDT: LazyStatic<Gdt> = LazyStatic::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
    Gdt {
        gdt,
        code_selector,
        tss_selector,
    }
});

struct Gdt {
    gdt: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Initialize the GlobalDescriptorTable
pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    GDT.gdt.load();
    unsafe {
        CS::set_reg(GDT.code_selector);
        load_tss(GDT.tss_selector);
    }
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: LazyStatic<TaskStateSegment> = LazyStatic::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(&raw const STACK);
        stack_start + STACK_SIZE as u64
    };
    tss
});
