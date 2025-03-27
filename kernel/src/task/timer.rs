use alloc::{
    boxed::Box,
    collections::{BTreeMap, btree_map::Entry},
};
use core::{
    sync::atomic::{AtomicU64, Ordering::*},
    task::{Poll, Waker},
};
use futures_util::Stream;
use spinlock::{DisableInterrupts, SpinLock};

static TIMER_WAKERS: SpinLock<BTreeMap<u64, NestedWaker>, DisableInterrupts> =
    SpinLock::disable_interrupts(BTreeMap::new());
pub static MILLIS: AtomicU64 = AtomicU64::new(0);

struct NestedWaker {
    waker: Waker,
    next: Option<Box<NestedWaker>>,
}

pub fn tick_ms() {
    let timer = MILLIS.fetch_add(1, Relaxed);
    let mut wakers = TIMER_WAKERS.lock();
    let Some((&timespamp, _)) = wakers.first_key_value() else {
        return;
    };
    if timespamp > timer {
        return;
    };
    let (_, NestedWaker { waker, mut next }) = wakers.pop_first().unwrap();
    waker.wake();
    while let Some(NestedWaker { waker, next: node }) = next.map(|b| *b) {
        waker.wake();
        next = node;
    }
}

pub struct Interval {
    last: u64,
    interval: u64,
}

impl Interval {
    pub fn new(ms: u64) -> Self {
        Self {
            last: MILLIS.load(Relaxed),
            interval: ms,
        }
    }
    pub fn catch_up(&mut self) {
        self.last = MILLIS.load(Relaxed);
    }
}

impl Stream for Interval {
    type Item = u64;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let timestamp = self.last + self.interval;
        if MILLIS.load(Relaxed) >= timestamp {
            self.last = timestamp;
            return Poll::Ready(Some(timestamp));
        }
        let waker = cx.waker().clone();
        match TIMER_WAKERS.lock().entry(timestamp) {
            Entry::Vacant(vacant) => {
                vacant.insert(NestedWaker { waker, next: None });
            }
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();
                let next = entry.next.take();
                let node = NestedWaker { waker, next };
                entry.next = Some(Box::new(node))
            }
        };
        if MILLIS.load(Relaxed) >= timestamp {
            self.last = timestamp;
            return Poll::Ready(Some(timestamp));
        }
        Poll::Pending
    }
}
