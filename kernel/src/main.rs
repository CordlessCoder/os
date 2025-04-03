#![feature(custom_test_frameworks)]
#![test_runner(kernel::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![no_std]
#![no_main]
extern crate alloc;
mod flappy;
mod snek;

use alloc::{format, string::String};
use bootloader::{BootInfo, entry_point};
use futures::{FutureExt, select_biased};
use futures_util::StreamExt;
use kernel::{
    prelude::{vga_color::*, *},
    task::{
        Task,
        executor::{Executor, Spawner},
        keyboard::KeypressStream,
        timer::Interval,
    },
    vga::{BUFFER_HEIGHT, BUFFER_WIDTH, ScreenChar},
};
use pc_keyboard::{DecodedKey, KeyCode, KeyEvent, KeyState};
use spinlock::LazyStatic;

const HELP_MESSAGE: &str = "Available commands:
snake - run snake
flappy / fb - run flappy bird
exit - exit the shell
help / ? - show this help message";

fn split_lines_and_wrap(text: &[u8], width: usize) -> impl DoubleEndedIterator<Item = &[u8]> {
    let lines = text.split(|&b| b == b'\n');
    lines.flat_map(move |line| {
        line.chunks(width).chain(core::iter::repeat_n(
            b"".as_slice(),
            if line.is_empty() { 1 } else { 0 },
        ))
    })
}

async fn main() {
    async fn print_mem_stats() {
        let mut timer = Interval::new(1000);
        loop {
            timer.tick().await;
            let stats = kernel::memory::global_alloc::ALLOCATOR.0.lock().stats();
            serial_println!("{stats:?}");
        }
    }
    select_biased! {
        _ = shell().fuse() => (),
        _ = print_mem_stats().fuse()=> ()
    }
}

async fn shell() {
    VGA_OUT.lock().enable_cursor(14, 15);
    async fn print_and_wait_for_input(keypresses: &mut KeypressStream, text: &str) {
        print(format!("{text}\nPress any button to return to shell.\n").as_bytes());
        loop {
            if let Some((_, Some(_))) = keypresses.next().await {
                break;
            };
        }
    }
    fn print(text: &[u8]) {
        let mut out = VGA_OUT.lock();
        out.fill_screen(b' ');
        let lines = split_lines_and_wrap(text, BUFFER_WIDTH);
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
    print(HELP_MESSAGE.as_bytes());
    VGA_OUT.lock().move_cursor(BUFFER_HEIGHT as u8 - 1, 0);
    while let Some((event, key)) = keypresses.next().await {
        print(buf.as_bytes());
        let end = split_lines_and_wrap(buf.as_bytes(), BUFFER_WIDTH)
            .last()
            .unwrap_or_default()
            .len();
        VGA_OUT
            .lock()
            .move_cursor(BUFFER_HEIGHT as u8 - 1, end as u8);
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
        let Some(key) = key else {
            continue;
        };
        match key {
            DecodedKey::Unicode('\n') if mods.is_shifted() => buf.push('\n'),
            DecodedKey::Unicode('\n') => {
                VGA_OUT.lock().disable_cursor();
                match buf.trim() {
                    "snek" | "snake" => snek::run().await,
                    "flappy" | "fb" => flappy::run().await,
                    "help" | "?" => print_and_wait_for_input(&mut keypresses, HELP_MESSAGE).await,
                    "exit" => return,
                    _ => print_and_wait_for_input(&mut keypresses, "No such command, type").await,
                }
                VGA_OUT.lock().enable_cursor(14, 15);
                buf.clear();
            }
            DecodedKey::Unicode(c) => buf.push(c),
            _ => (),
        }
    }
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

    println!(fgcolor = LightCyan, "Async executor exited successfully.");
    println!(fgcolor = White, "Please shut down the system.");
    kernel::hlt_loop()
}
