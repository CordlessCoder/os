pub mod executor;
pub mod keyboard;
pub mod timer;

use alloc::boxed::Box;
use core::{fmt::Debug, future::Future, pin::Pin, sync::atomic::AtomicU64};

/// A unique ID generated when a Task is created.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

type TaskInnerFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Represents a Future with a unique ID that can be scheduled on the Executor.
pub struct Task {
    id: TaskId,
    future: TaskInnerFuture,
}

impl Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("future", &core::any::type_name_of_val(&*self.future))
            .finish()
    }
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static + Send) -> Self {
        Task {
            future: Box::pin(future),
            id: TaskId::new(),
        }
    }
}

/// Initialize task dependencies.
pub fn init() {
    keyboard::SCANCODE_QUEUE.force();
}
