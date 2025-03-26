#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]

use bootloader::{BootInfo, entry_point};
use kernel::prelude::*;

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    kernel::enable_test();
    test_main();
    unreachable!()
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
