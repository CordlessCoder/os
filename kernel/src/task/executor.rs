use super::{Task, TaskId};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    task::Wake,
};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use spinlock::{DisableInterrupts, SpinLock};
use x86_64::instructions::interrupts;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    to_be_woken: Arc<SpinLock<BTreeSet<TaskId>, DisableInterrupts>>,
    wakers: BTreeMap<TaskId, Waker>,
    spawner: Arc<ArrayQueue<Task>>,
}

#[derive(Clone, Debug)]
pub struct Spawner {
    queue: Arc<ArrayQueue<Task>>,
}

impl Spawner {
    pub fn spawn_task(&self, task: Task) {
        if self.queue.push(task).is_err() {
            panic!(
                "The spawner queue of the executor is full, the executor does not seem to be polling the spawner."
            )
        };
    }
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let task = Task {
            future: Box::pin(future),
            id: TaskId::new(),
        };
        self.spawn_task(task);
    }
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            wakers: BTreeMap::new(),
            spawner: Arc::new(ArrayQueue::new(64)),
            to_be_woken: Arc::new(SpinLock::disable_interrupts(BTreeSet::new())),
        }
    }
    pub fn spawner(&self) -> Spawner {
        let queue = self.spawner.clone();
        Spawner { queue }
    }
    pub fn spawn(&mut self, task: Task) {
        let id = task.id;
        if self.tasks.insert(id, task).is_some() {
            panic!("Task with {id:?} already in tasks.");
        }
        self.to_be_woken.lock().insert(id);
    }
    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
    pub fn has_woken_tasks(&self) -> bool {
        !self.to_be_woken.lock().is_empty()
    }
    pub fn poll_spawner(&mut self) {
        while let Some(task) = self.spawner.pop() {
            self.spawn(task);
        }
    }
    /// Returns false if there are no tasks to run, false otherwise
    pub fn poll_one(&mut self) -> bool {
        self.poll_spawner();
        let Self {
            tasks,
            wakers,
            to_be_woken,
            ..
        } = self;
        let Some(id) = to_be_woken.lock().pop_first() else {
            return false;
        };
        let Some(task) = tasks.get_mut(&id) else {
            // Task no longer exists
            return true;
        };
        let waker = wakers
            .entry(id)
            .or_insert_with(|| TaskWaker::new_waker(id, to_be_woken.clone()));
        let mut cx = Context::from_waker(waker);
        match task.poll(&mut cx) {
            Poll::Ready(()) => {
                tasks.remove(&id);
                wakers.remove(&id);
            }
            Poll::Pending => (),
        };
        true
    }
    /// Will run the executor until all tasks exit
    pub fn run(&mut self) {
        self.poll_spawner();
        while self.has_tasks() {
            while self.poll_one() {}
            if self.has_tasks() {
                self.sleep_if_idle();
            }
            self.poll_spawner();
        }
    }
    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.to_be_woken.lock().is_empty() {
            interrupts::enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    id: TaskId,
    to_be_woken: Arc<SpinLock<BTreeSet<TaskId>, DisableInterrupts>>,
}

impl TaskWaker {
    fn new_waker(
        id: TaskId,
        to_be_woken: Arc<SpinLock<BTreeSet<TaskId>, DisableInterrupts>>,
    ) -> Waker {
        Waker::from(Arc::new(TaskWaker { id, to_be_woken }))
    }
    fn wake_task(&self) {
        self.to_be_woken.lock().insert(self.id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}
