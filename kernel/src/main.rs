#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
use bootloader::{BootInfo, entry_point};
use kernel::{
    memory,
    prelude::{vga_color::*, *},
};
use x86_64::structures::paging::Translate;

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    kernel::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { kernel::memory::get_table(phys_offset) };
    let _frame_alloc =
        unsafe { memory::frame_alloc::BootInfoFrameAllocator::new(&boot_info.memory_map) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    println!(fgcolor = LightCyan, "We didn't crash!");
    kernel::hlt_loop()
}
