use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts;

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    queue: Arc<ArrayQueue<TaskId>>,
    wakers: BTreeMap<TaskId, Waker>,
}

impl Executor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Executor {
        Executor {
            queue: Arc::new(ArrayQueue::new(64)),
            tasks: BTreeMap::new(),
            wakers: BTreeMap::new(),
        }
    }
    pub fn spawn(&mut self, task: Task) {
        let id = task.id;
        if self.tasks.insert(id, task).is_some() {
            panic!("Task with {id:?} already in tasks.");
        }
        self.queue.push(id).expect("queue full");
    }
    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
    pub fn has_woken_tasks(&self) -> bool {
        !self.queue.is_empty()
    }
    /// Returns false if there are no tasks to run, false otherwise
    pub fn poll_one(&mut self) -> bool {
        let Self {
            tasks,
            queue,
            wakers,
        } = self;
        let Some(id) = queue.pop() else {
            return false;
        };
        let Some(task) = tasks.get_mut(&id) else {
            // Task no longer exists
            return true;
        };
        let waker = wakers
            .entry(id)
            .or_insert_with(|| TaskWaker::new_waker(id, queue.clone()));
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
        loop {
            while self.has_woken_tasks() {
                self.poll_one();
            }
            self.sleep_if_idle();
        }
    }
    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.queue.is_empty() {
            interrupts::enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    id: TaskId,
    queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new_waker(id: TaskId, queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker { id, queue }))
    }
    fn wake_task(&self) {
        self.queue.push(self.id).expect("task queue full")
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
