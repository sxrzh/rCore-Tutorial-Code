//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
// use alloc::collections::VecDeque;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use lazy_static::*;

/// use BinaryHeap or VecDeque to implement priority queue
struct PriorityQueueWrapper<T> {
    data: BinaryHeap<T>,
}
impl<T: Ord> PriorityQueueWrapper<T> {
    pub fn new() -> Self {
        Self {
            data: BinaryHeap::<T>::new(),
        }
    }
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn push(&mut self, item: T) {
        self.data.push(item)
    }
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    #[allow(unused)]
    pub fn peek(&self) -> Option<&T> {
        self.data.peek()
    }
}

///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: PriorityQueueWrapper<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: PriorityQueueWrapper::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if let Some(task) = self.ready_queue.pop() {
            task.stride_step();
            Some(task)
        }
        else {
            None
        }
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    // trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
