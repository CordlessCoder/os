use core::task::Poll;

use crate::prelude::{vga_color::*, *};
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, task::AtomicWaker};
use spinlock::LazyStatic;

pub static SCANCODE_QUEUE: LazyStatic<ArrayQueue<u8>> = LazyStatic::new(|| ArrayQueue::new(64));
static SCANCODE_WAKER: AtomicWaker = AtomicWaker::new();

pub fn add_scancode(scancode: u8) {
    let Some(queue) = SCANCODE_QUEUE.get_if_init() else {
        println!(fgcolor = Red, "WARNING: SCANCODE_QUEUE not initialized.");
        return;
    };
    if queue.push(scancode).is_err() {
        println!(
            fgcolor = Red,
            "WARNING: SCANCODE_QUEUE is full, dropping scancode"
        );
        return;
    }
    SCANCODE_WAKER.wake();
}

struct ScancodeStream(());

impl ScancodeStream {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SCANCODE_QUEUE.force();
        Self(())
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.force();
        if let Some(s) = queue.pop() {
            return Poll::Ready(Some(s));
        };
        SCANCODE_WAKER.register(cx.waker());
        match queue.pop() {
            Some(s) => {
                SCANCODE_WAKER.take();
                Poll::Ready(Some(s))
            }
            None => Poll::Pending,
        }
    }
}
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!(fgcolor = Blue, "{}", character),
                    DecodedKey::RawKey(key) => print!(fgcolor = LightBlue, "{:?}", key),
                }
            }
        }
    }
}
