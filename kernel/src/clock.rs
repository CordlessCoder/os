use core::{
    sync::atomic::{AtomicU64, Ordering::*},
    time::Duration,
};

/// A monotonically-nondecreasing millisecond-granular clock
///
/// Gets incremented every millisecond by the Programmable Interrupt Timer
pub(crate) static MS_CLOCK: AtomicU64 = AtomicU64::new(0);

/// Advances the global clock by 1 ms
pub fn tick_ms() {
    let timer = MS_CLOCK.fetch_add(1, Relaxed);
    crate::task::timer::wake_tasks(timer);
}

fn load_now() -> u64 {
    MS_CLOCK.load(Relaxed)
}

/// Represents a measurement of the global millisecond-granular clock.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);

impl Instant {
    pub fn now() -> Self {
        Self(load_now())
    }
    pub fn since_epoch(&self) -> u64 {
        self.0
    }
    pub fn elapsed_ms(&self) -> u64 {
        load_now().wrapping_sub(self.0)
    }
    pub fn elapsed(&self) -> Duration {
        let ms = self.elapsed_ms();
        Duration::from_millis(ms)
    }
    pub fn duration_since(&self, Self(earlier): Self) -> Duration {
        let later = self.since_epoch();
        let difference = later.saturating_sub(earlier);
        Duration::from_millis(difference)
    }
    pub fn checked_add_ms(&self, ms: u64) -> Option<Self> {
        self.0.checked_add(ms).map(Self::from)
    }
    pub fn checked_sub_ms(&self, ms: u64) -> Option<Self> {
        self.0.checked_sub(ms).map(Self::from)
    }
    pub fn checked_add(&self, dur: Duration) -> Option<Self> {
        let ms: u64 = dur.as_millis().try_into().ok()?;
        self.checked_add_ms(ms)
    }
    pub fn checked_sub(&self, dur: Duration) -> Option<Self> {
        let ms: u64 = dur.as_millis().try_into().ok()?;
        self.checked_sub_ms(ms)
    }
}

impl From<u64> for Instant {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
