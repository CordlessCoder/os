#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(doom_from_scratch::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_main]

use doom_from_scratch::prelude::*;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    unreachable!()
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
