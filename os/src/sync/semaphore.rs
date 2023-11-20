//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{
    block_current_and_run_next, current_task, current_task_id, wakeup_task, TaskControlBlock,
};
use alloc::collections::{BTreeSet, VecDeque};
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::{Resource, Tid};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub allocated_to: BTreeSet<Tid>,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    allocated_to: BTreeSet::new(),
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
        trace!("kernel: Semaphore::up");
        let mut inner = self.inner.exclusive_access();
        // remove current tid from allocated_to
        let tid = current_task_id().unwrap();
        inner.allocated_to.remove(&tid);
        inner.count += 1;
        if inner.count <= 0 {
            if let Some(task) = inner.wait_queue.pop_front() {
                wakeup_task(task);
            }
        }
    }

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        } else {
            let tid = current_task_id().unwrap();
            inner.allocated_to.insert(tid);
        }
    }
}

impl Resource for Semaphore {
    fn get_available(&self) -> usize {
        match self.inner.exclusive_access().count {
            n if n < 0 => 0,
            n => n as usize,
        }
    }

    fn get_allocation(&self) -> Vec<(Tid, usize)> {
        let inner = self.inner.exclusive_access();
        inner.allocated_to.iter().map(|tid| (*tid, 1)).collect()
    }

    fn get_need(&self) -> Vec<(Tid, usize)> {
        let inner = self.inner.exclusive_access();
        inner
            .wait_queue
            .iter()
            .map(|tcb| (tcb.inner_exclusive_access().res.as_ref().unwrap().tid, 1))
            .collect()
    }
}
