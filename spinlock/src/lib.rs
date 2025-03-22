#![no_std]
use core::ops::{Deref, DerefMut};
use core::sync::atomic::Ordering::*;
use core::{cell::UnsafeCell, sync::atomic::AtomicBool};
pub mod lazylock;
pub mod lazystatic;

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

// SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
pub struct Guard<'l, T> {
    lock: &'l SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }
    pub fn try_lock(&self) -> Option<Guard<'_, T>> {
        let aquired = !self.locked.swap(true, Acquire);
        aquired.then_some(Guard { lock: self })
    }
    pub fn lock(&self) -> Guard<'_, T> {
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

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:The existence of a guard proves that we have successfully acquired the SpinLock
        unsafe { &mut *self.lock.value.get() }
    }
}
unsafe impl<T> Send for Guard<'_, T> where T: Send {}
unsafe impl<T> Sync for Guard<'_, T> where T: Sync {}

impl<T> Guard<'_, T> {
    pub fn unlock(self) {}
}
