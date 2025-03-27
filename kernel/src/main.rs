#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
extern crate alloc;

use bootloader::{BootInfo, entry_point};
use kernel::{
    prelude::{vga_color::*, *},
    task::{Task, executor::Executor, keyboard},
};
// use x86_64::structures::paging::Translate;

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    #[cfg(test)]
    kernel::enable_test();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    println!(fgcolor = LightCyan, "We didn't crash!");
    kernel::hlt_loop()
}
