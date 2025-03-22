#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
use kernel::prelude::{vga_color::*, *};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    kernel::init();

    #[cfg(test)]
    test_main();

    // trigger a page fault
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };

    println!(fgcolor = LightCyan, "We didn't crash!");
    todo!()
}
