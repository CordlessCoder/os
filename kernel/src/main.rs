#![feature(custom_test_frameworks)]
#![test_runner(doom_from_scratch::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
use doom_from_scratch::prelude::{vga_color::*, *};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    doom_from_scratch::init();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3(); // new

    #[cfg(test)]
    test_main();

    println!(
        fgcolor = LightBlue,
        "The numbers are {} and {}",
        42,
        1.0 / 3.0
    );
    serial_println!("Have a serial");

    println!(fgcolor = LightCyan, "Hello, world!");
    todo!()
}
