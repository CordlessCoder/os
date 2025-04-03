use core::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    ops::Deref,
    sync::atomic::{AtomicU8, Ordering::*},
};

/// A wrapper for on-demand *one-time* initialization of a value.
pub struct LazyStatic<T, F = fn() -> T> {
    // 0 = Uninit
    // 1 = In Progress
    // 2 = Init
    state: AtomicU8,
    storage: UnsafeCell<Storage<T, F>>,
}
unsafe impl<T: Sync, F> Sync for LazyStatic<T, F> {}
unsafe impl<T: Sync, F> Send for LazyStatic<T, F> {}

union Storage<T, F> {
    compute: ManuallyDrop<F>,
    data: ManuallyDrop<T>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InitStatus {
    Uninit = 0,
    InProgress,
    Init,
}

impl<T, F: FnOnce() -> T> LazyStatic<T, F> {
    pub const fn new(compute: F) -> Self {
        Self {
            state: AtomicU8::new(0),
            storage: UnsafeCell::new(Storage {
                compute: ManuallyDrop::new(compute),
            }),
        }
    }
    pub fn status(&self) -> InitStatus {
        match self.state.load(Acquire) {
            0 => InitStatus::Uninit,
            1 => InitStatus::InProgress,
            2 => InitStatus::Init,
            _ => unreachable!(),
        }
    }
    pub fn get_if_init(&self) -> Option<&T> {
        (self.status() == InitStatus::Init)
            .then(|| unsafe { &*self.storage.get().as_ref().unwrap_unchecked().data })
    }
    pub fn insert_if_uninit(&self, val: T) -> Result<(), T> {
        if self.state.compare_exchange(0, 1, Release, Acquire).is_err() {
            return Err(val);
        }
        // SAFETY: At this point, state has been set to 1(In progress)
        // and self.storage must hold a compute
        unsafe {
            // SAFETY: Materializing this reference is safe as we have locked the
            // state and therefore no other threads will attempt to access self.storage
            let storage = &mut *self.storage.get();
            // SAFETY: The transition from 0(Uninit) -> 1(In Progress) only happens
            // once, so we're allowed to take out the value
            ManuallyDrop::drop(&mut storage.compute);
            storage.data = ManuallyDrop::new(val);
            // Release the data to all the other threads
            self.state.store(2, Release);
        }
        Ok(())
    }
    /// Force the inner value to be computed, and get a reference to it.
    pub fn force(&self) -> &T {
        loop {
            match self.state.compare_exchange(0, 1, Release, Acquire) {
                Ok(_) => {
                    // We need to run compute
                    // SAFETY: At this point, state has been set to 1(In progress)
                    // and self.storage must hold a compute
                    unsafe {
                        // SAFETY: Materializing this reference is safe as we have locked the
                        // state and therefore no other threads will attempt to access self.storage
                        let storage = &mut *self.storage.get();
                        // SAFETY: The transition from 0(Uninit) -> 1(In Progress) only happens
                        // once, so we're allowed to take out the value
                        let compute = ManuallyDrop::take(&mut storage.compute);
                        let value = compute();
                        // Release the data to all the other threads
                        storage.data = ManuallyDrop::new(value);
                        self.state.store(2, Release);
                        break;
                    }
                }
                Err(2) => {
                    // The value has already been computed
                    break;
                }
                Err(1) => {
                    // Another thread captured to compute
                    core::hint::spin_loop();
                }
                Err(_) => unreachable!(),
            }
        }
        // SAFETY: The only way to reach this line is by issuing a break in the above loop.
        // This only occurs if the value has already been computed by another thread, or we just
        // finished computing it ourselves.
        unsafe { &self.storage.get().as_ref().unwrap_unchecked().data }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyStatic<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.force()
    }
}

impl<T, F> Drop for LazyStatic<T, F> {
    fn drop(&mut self) {
        match self.state.load(Acquire) {
            0 => {
                // Uninit
                unsafe {
                    ManuallyDrop::drop(&mut self.storage.get_mut().compute);
                }
            }
            2 => {
                // Init
                unsafe {
                    ManuallyDrop::drop(&mut self.storage.get_mut().data);
                }
            }
            _ => {
                // Poisoned
            }
        }
    }
}
