#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
extern crate alloc;

use bootloader::{BootInfo, entry_point};
use futures_util::StreamExt;
use kernel::{
    prelude::{vga_color::*, *},
    task::{Task, executor::Executor, keyboard::Keypresses, timer::Ticks},
};
use pc_keyboard::DecodedKey;

async fn print_keypresses() {
    let mut keypresses = Keypresses::new();
    loop {
        let key = keypresses.next_keypress().await;
        match key {
            DecodedKey::Unicode(character) => println!(fgcolor = Blue, "{}", character),
            DecodedKey::RawKey(key) => println!(fgcolor = LightBlue, "{:?}", key),
        }
    }
}
async fn print_ticks() {
    let mut ticks = Ticks::new();
    while let Some(tick) = ticks.next().await {
        println!(fgcolor = Yellow, "Tick {tick}!");
    }
}

entry_point!(main);
fn main(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    #[cfg(test)]
    kernel::enable_test();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(print_keypresses()));
    executor.spawn(Task::new(print_ticks()));
    executor.run();

    println!(fgcolor = LightCyan, "Executor exited successfully");
    kernel::hlt_loop()
}
