use core::{
    cell::{LazyCell, UnsafeCell},
    ops::{Deref, DerefMut},
};

use crate::{SpinLock, SpinLockGuard};

pub struct LazyLock<T, F = fn() -> UnsafeCell<T>>(SpinLock<LazyCell<UnsafeCell<T>, F>>);
pub struct LazyLockGuard<'l, T, F>(SpinLockGuard<'l, LazyCell<UnsafeCell<T>, F>>);

impl<T, F: FnOnce() -> UnsafeCell<T>> LazyLock<T, F> {
    pub const fn new(compute: F) -> Self {
        Self(SpinLock::new(LazyCell::new(compute)))
    }
    pub fn lock<'l>(&'l self) -> LazyLockGuard<'l, T, F> {
        let guard = self.0.lock();
        LazyLockGuard(guard)
    }
    /// # Safety
    /// This does not lock the mutex.
    /// Only use this if you can otherwise prove that the mutex is not locked.
    pub const unsafe fn get_inner_mut(&self) -> *mut LazyCell<UnsafeCell<T>, F> {
        unsafe { self.0.get_inner_mut() }
    }
}

impl<T, F: FnOnce() -> UnsafeCell<T>> Deref for LazyLockGuard<'_, T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let cell = self.0.deref();
        let cell = LazyCell::force(cell);
        // SAFETY: This reference only lives as long as the lock guard locking the entire LazyCell
        unsafe { &*cell.get() }
    }
}
impl<T, F: FnOnce() -> UnsafeCell<T>> DerefMut for LazyLockGuard<'_, T, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let cell = self.0.deref();
        let cell = LazyCell::force(cell);
        // SAFETY: This reference only lives as long as the lock guard locking the entire LazyCell
        unsafe { &mut *cell.get() }
    }
}
