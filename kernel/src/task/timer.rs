use crate::prelude::{vga_color::*, *};
use core::{
    sync::atomic::{AtomicU64, Ordering::*},
    task::Poll,
};
use futures_util::{Stream, StreamExt, task::AtomicWaker};
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts::Us104Key};

static TIMER_WAKER: AtomicWaker = AtomicWaker::new();
static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn tick() {
    TICKS.fetch_add(1, Relaxed);
    TIMER_WAKER.wake();
}

pub struct Ticks {
    current: u64,
}

impl Ticks {
    pub const fn from_epoch() -> Self {
        Self { current: 0 }
    }
    pub fn new() -> Self {
        let mut timer = Self::from_epoch();
        timer.catch_up();
        timer
    }
    pub fn catch_up(&mut self) {
        self.current = TICKS.load(Relaxed);
    }
}

impl Default for Ticks {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for Ticks {
    type Item = u64;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        if TICKS.load(Relaxed) > self.current {
            self.current += 1;
            return Poll::Ready(Some(self.current));
        }
        TIMER_WAKER.register(cx.waker());
        if TICKS.load(Relaxed) > self.current {
            self.current += 1;
            return Poll::Ready(Some(self.current));
        }
        Poll::Pending
    }
}
