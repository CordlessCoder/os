#![no_std]
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::option::Option::{self, *};
use core::sync::atomic::Ordering::*;
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};
mod lazystatic;
pub use lazystatic::*;
#[cfg(feature = "x86_64_disable_interrupts")]
use x86_64::instructions::interrupts;

pub trait InterruptHandlingStrategy {
    type RestoreState;
    fn apply(&self) -> Self::RestoreState;
    fn restore_state(&self, state: Self::RestoreState);
}

#[cfg(feature = "x86_64_disable_interrupts")]
pub struct DisableInterrupts;
#[cfg(feature = "x86_64_disable_interrupts")]
impl InterruptHandlingStrategy for DisableInterrupts {
    type RestoreState = bool;
    fn apply(&self) -> Self::RestoreState {
        let enabled = interrupts::are_enabled();
        interrupts::disable();
        enabled
    }
    fn restore_state(&self, state: Self::RestoreState) {
        if state {
            interrupts::enable();
        }
    }
}
pub struct KeepInterrupts;
impl InterruptHandlingStrategy for KeepInterrupts {
    type RestoreState = ();
    fn apply(&self) -> Self::RestoreState {}
    fn restore_state(&self, _state: Self::RestoreState) {}
}

pub struct SpinLock<T, IH: InterruptHandlingStrategy = KeepInterrupts> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
    ih: IH,
}
unsafe impl<T, IH: InterruptHandlingStrategy> Sync for SpinLock<T, IH> where T: Send {}

// SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
pub struct SpinLockGuard<'l, T, IH: InterruptHandlingStrategy> {
    lock: &'l SpinLock<T, IH>,
    interrupt_state: MaybeUninit<IH::RestoreState>,
}

impl<T> SpinLock<T, KeepInterrupts> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self::with_ih(KeepInterrupts, value)
    }
}
#[cfg(feature = "x86_64_disable_interrupts")]
impl<T> SpinLock<T, DisableInterrupts> {
    #[inline]
    pub const fn disable_interrupts(value: T) -> Self {
        Self::with_ih(DisableInterrupts, value)
    }
}
impl<T, IH: InterruptHandlingStrategy> SpinLock<T, IH> {
    #[inline]
    pub const fn with_ih(ih: IH, value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
            ih,
        }
    }
    #[inline]
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T, IH>> {
        let interrupt_state = self.ih.apply();
        if self.locked.swap(true, Acquire) {
            self.ih.restore_state(interrupt_state);
            return None;
        }
        Some(SpinLockGuard {
            lock: self,
            interrupt_state: MaybeUninit::new(interrupt_state),
        })
    }
    pub fn locked(&self) -> bool {
        self.locked.load(Acquire)
    }
    pub fn lock(&self) -> SpinLockGuard<'_, T, IH> {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            };
            core::hint::spin_loop();
        }
    }
    /// # Safety
    /// This does not lock the mutex.
    /// Only use this if you can otherwise prove that the mutex is not locked.
    pub const unsafe fn get_inner_mut(&self) -> *mut T {
        self.value.get()
    }
}

impl<T, IH: InterruptHandlingStrategy> Drop for SpinLockGuard<'_, T, IH> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
        self.lock
            .ih
            .restore_state(unsafe { MaybeUninit::assume_init_read(&self.interrupt_state) });
    }
}

impl<T, IH: InterruptHandlingStrategy> Deref for SpinLockGuard<'_, T, IH> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
        unsafe { &*self.lock.value.get() }
    }
}

impl<T, IH: InterruptHandlingStrategy> DerefMut for SpinLockGuard<'_, T, IH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
        unsafe { &mut *self.lock.value.get() }
    }
}
unsafe impl<T, IH: InterruptHandlingStrategy> Send for SpinLockGuard<'_, T, IH>
where
    T: Send,
    IH: Send,
{
}
unsafe impl<T, IH: InterruptHandlingStrategy> Sync for SpinLockGuard<'_, T, IH> where T: Sync {}

impl<T, IH: InterruptHandlingStrategy> SpinLockGuard<'_, T, IH> {
    pub fn unlock(self) {}
}
