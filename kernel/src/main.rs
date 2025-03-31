#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
extern crate alloc;
mod flappy;
mod snek;

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
    vga::{BUFFER_WIDTH, ScreenChar},
};
use pc_keyboard::{DecodedKey, KeyCode, KeyEvent, KeyState};
use spinlock::LazyStatic;

async fn main() {
    fn print(buf: &[u8]) {
        let mut out = VGA_OUT.lock();
        out.fill_screen(b' ');
        let lines = buf.split(|&b| b == b'\n');
        let lines = lines.flat_map(|line| {
            line.chunks(BUFFER_WIDTH).chain(core::iter::repeat_n(
                b"".as_slice(),
                if line.is_empty() { 1 } else { 0 },
            ))
        });
        let lines = [[b'-'; BUFFER_WIDTH].as_slice()].into_iter().chain(lines);
        let color = out.color;
        out.buf.map_framebuffer(move |mut buf| {
            for (target, text) in buf.iter_mut().rev().zip(lines.rev()) {
                *target = [ScreenChar { ascii: b' ', color }; 80];
                target
                    .iter_mut()
                    .zip(text)
                    .for_each(|(out, &ascii)| *out = ScreenChar { ascii, color });
            }
            buf
        });
    }
    let mut keypresses = KeypressStream::new();
    let mut buf = String::new();
    print(buf.as_bytes());
    while let Some((event, key)) = keypresses.next().await {
        print(buf.as_bytes());
        let mods = keypresses.keyboard.get_modifiers();
        match event {
            KeyEvent {
                code: KeyCode::Backspace,
                state: KeyState::Down,
            } => {
                buf.pop();
                continue;
            }
            KeyEvent {
                code: KeyCode::C, ..
            } if mods.is_ctrl() => {
                buf.clear();
                continue;
            }
            _ => (),
        }
        match key {
            Some(DecodedKey::Unicode('\n')) if mods.is_shifted() => buf.push('\n'),
            Some(DecodedKey::Unicode('\n')) => {
                if buf.trim().eq_ignore_ascii_case("snek") {
                    snek::run().await;
                }
                if buf.trim().eq_ignore_ascii_case("flappy") {
                    flappy::run().await;
                }
                if buf.trim().eq_ignore_ascii_case("exit") {
                    return;
                }
                buf.clear();
            }
            Some(DecodedKey::Unicode(c)) => buf.push(c),
            _ => (),
        }
    }
}

static SPAWNER: LazyStatic<Spawner> =
    LazyStatic::new(|| panic!("Attempted to use spawner before initializing executor"));

async fn print_mem_stats() {
    let mut timer = Interval::new(1000);
    loop {
        timer.tick().await;
        let stats = kernel::memory::global_alloc::ALLOCATOR.0.lock().stats();
        serial_println!("{stats:?}");
    }
}

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
    executor.spawn(Task::new(print_mem_stats()));
    executor.run();

    println!(fgcolor = LightCyan, "Executor exited successfully");
    kernel::hlt_loop()
}
