use crate::clock::MS_CLOCK;
use alloc::{
    boxed::Box,
    collections::{BTreeMap, btree_map::Entry},
};
use core::{
    num::NonZeroU64,
    sync::atomic::Ordering::*,
    task::{Poll, Waker},
};
use futures_util::{Stream, StreamExt};
use spinlock::{DisableInterrupts, SpinLock};

/// The wakers registered to fire when the
static TIMER_WAKERS: SpinLock<BTreeMap<u64, NestedWaker>, DisableInterrupts> =
    SpinLock::disable_interrupts(BTreeMap::new());

struct NestedWaker {
    waker: Waker,
    next: Option<Box<NestedWaker>>,
}

/// Wake any tasks registered to fire before the provided timestamp of the MS_CLOCK.
pub fn wake_tasks(timestamp: u64) {
    let mut wakers = TIMER_WAKERS.lock();
    loop {
        let Some((&timespamp, _)) = wakers.first_key_value() else {
            break;
        };
        if timespamp > timestamp {
            break;
        };
        let (_, NestedWaker { waker, mut next }) = wakers.pop_first().unwrap();
        waker.wake();
        while let Some(NestedWaker { waker, next: node }) = next.map(|b| *b) {
            waker.wake();
            next = node;
        }
    }
}

pub struct Interval {
    last: u64,
    interval: NonZeroU64,
}

impl Interval {
    pub fn new(ms: u64) -> Self {
        Self {
            last: MS_CLOCK.load(Relaxed).wrapping_sub(ms),
            interval: ms
                .try_into()
                .expect("Cannot create an interval that yields every 0 ms."),
        }
    }
    pub fn reset(&mut self) {
        self.last = MS_CLOCK.load(Relaxed).wrapping_sub(self.interval.get());
    }
    pub async fn tick(&mut self) {
        self.next().await;
    }
}

pub async fn sleep(ms: u64) {
    sleep_until(MS_CLOCK.load(Relaxed) + ms).await
}

pub async fn sleep_until(timestamp: u64) {
    let Some(interval) = NonZeroU64::new(timestamp) else {
        return;
    };
    Interval { last: 0, interval }.tick().await;
}

impl Stream for Interval {
    type Item = u64;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        let timestamp = self.last.wrapping_add(self.interval.get());
        if MS_CLOCK.load(Relaxed) >= timestamp {
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
        if MS_CLOCK.load(Relaxed) >= timestamp {
            self.last = timestamp;
            return Poll::Ready(Some(timestamp));
        }
        Poll::Pending
    }
}
