#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::String;
use bootloader::{BootInfo, entry_point};
use futures_util::StreamExt;
use kernel::{
    prelude::{vga_color::*, *},
    task::{
        Task,
        executor::{Executor, Spawner},
        keyboard::KeypressStream,
        timer::Interval,
    },
};
use pc_keyboard::DecodedKey;
use spinlock::LazyStatic;

async fn print_keypresses() {
    let mut keypresses = KeypressStream::new();
    while let Some(key) = keypresses.next().await {
        match key {
            DecodedKey::Unicode(character) => println!(fgcolor = Blue, "{}", character),
            DecodedKey::RawKey(key) => println!(fgcolor = LightBlue, "{:?}", key),
        }
    }
}
async fn print_every_second() {
    let mut ticks = Interval::new(1000);
    while let Some(tick) = ticks.next().await {
        println!(fgcolor = Yellow, "Tick {tick}!");
        kernel::memory::global_alloc::ALLOCATOR.0.lock().debug();
    }
}

async fn main() {
    // let mut keypresses = Keypresses::new();
    // keypresses.next_keypress()
    // let mut buf = String::new();
    SPAWNER.spawn(print_keypresses());
    SPAWNER.spawn(print_every_second());
}

static SPAWNER: LazyStatic<Spawner> =
    LazyStatic::new(|| panic!("Attempted to use spawner before initializing executor"));

entry_point!(entrypoint);
fn entrypoint(boot_info: &'static BootInfo) -> ! {
    kernel::init(boot_info);
    #[cfg(test)]
    kernel::enable_test();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    SPAWNER.insert_if_uninit(executor.spawner()).unwrap();
    executor.spawn(Task::new(main()));
    executor.run();

    println!(fgcolor = LightCyan, "Executor exited successfully");
    kernel::hlt_loop()
}
