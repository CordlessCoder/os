use core::task::Poll;

use crate::prelude::{vga_color::*, *};
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts::Us104Key};
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
pub struct Keypresses {
    keyboard: Keyboard<Us104Key, ScancodeSet1>,
    scancode_stream: ScancodeStream,
}

impl Keypresses {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let keyboard = Keyboard::new(
            ScancodeSet1::new(),
            Us104Key,
            pc_keyboard::HandleControl::Ignore,
        );
        let scancode_stream = ScancodeStream::new();
        Self {
            keyboard,
            scancode_stream,
        }
    }
    pub async fn next_keypress(&mut self) -> DecodedKey {
        loop {
            let event = loop {
                let sc = self
                    .scancode_stream
                    .next()
                    .await
                    .expect("Scancode stream is empty");
                if let Ok(Some(event)) = self.keyboard.add_byte(sc) {
                    break event;
                }
            };
            if let Some(key) = self.keyboard.process_keyevent(event) {
                return key;
            }
        }
    }
}
