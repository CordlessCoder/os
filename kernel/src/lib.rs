#![feature(abi_x86_interrupt, custom_test_frameworks)]
#![allow(clippy::empty_loop)]
#![no_std]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
pub mod clock;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod panic;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod test;
pub mod vga;
pub mod prelude {
    pub use crate::serial::SERIAL1;
    pub use crate::vga::{
        ColorCode, VGA_OUT,
        color::{Blink, Color, LightColor},
    };
    pub mod vga_color {
        use super::{Color, LightColor};
        pub use Color::*;
        pub use LightColor::*;
    }
    pub use crate::{print, println, serial_print, serial_println};
}
use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    gdt::init();
    vga::init();
    interrupts::init();
    memory::init(boot_info);
    task::init();
}

/// Redirect panic output to QEMU's serial/stdout.
pub fn enable_test() {
    panic::set_qemu_test_panic();
}

#[cfg(test)]
use bootloader::entry_point;
#[cfg(test)]
entry_point!(main);
/// Entry point for `cargo test`
#[cfg(test)]
#[unsafe(no_mangle)]
pub fn main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
