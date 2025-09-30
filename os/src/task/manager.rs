//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Ordering;
use lazy_static::*;

/// P.pass = BigStride / P.priority
/// todo: choose a robust value and comparison
const BIG_STRIDE: usize = 0xFFFFFFF;

/// A wrapper for comparing stride promising exclusive access
struct TaskWrapper(Arc<TaskControlBlock>);

impl PartialEq for TaskWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.get_stride() == other.0.get_stride()
    }
}

impl Eq for TaskWrapper {}

impl PartialOrd for TaskWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0
            .get_stride()
            .partial_cmp(&other.0.get_stride())
            .map(|ord| match ord {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            })
    }
}

impl Ord for TaskWrapper {
    /// Task with lesser stride is treated as higher priority in priority queue
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.0.get_stride().cmp(&other.0.get_stride()) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

/// A thread-safe priority queue
pub struct TaskManager {
    ready_queue: BinaryHeap<TaskWrapper>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
        }
    }
    /// Add  process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(TaskWrapper(task));
    }
    /// Take the process with the least stride out of the ready queue and update its stride
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop().map(|TaskWrapper(task)| {
            task.update_stride(BIG_STRIDE / task.get_priority());
            task
        })
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
