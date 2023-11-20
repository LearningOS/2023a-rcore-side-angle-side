//! Mutex (spin-like and blocking(sleep))

use super::{Resource, Tid, UPSafeCell};
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use crate::task::{current_task_id, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};
use alloc::{vec, vec::Vec};

/// Mutex trait
pub trait Mutex: Sync + Send + Resource {
    /// Lock the mutex
    fn lock(&self);
    /// Unlock the mutex
    fn unlock(&self);
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    inner: UPSafeCell<MutexSpinInner>,
}

pub struct MutexSpinInner {
    locked: bool,
    allocated_to: Tid,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexSpinInner {
                    locked: false,
                    allocated_to: 0,
                })
            },
        }
    }
}

impl Mutex for MutexSpin {
    /// Lock the spinlock mutex
    fn lock(&self) {
        trace!("kernel: MutexSpin::lock");
        loop {
            let mut inner = self.inner.exclusive_access();
            if inner.locked {
                drop(inner);
                suspend_current_and_run_next();
                continue;
            } else {
                inner.locked = true;
                let tid = current_task_id().unwrap();
                inner.allocated_to = tid;
                return;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        let mut inner = self.inner.exclusive_access();
        inner.locked = false;
    }
}

impl Resource for MutexSpin {
    fn get_available(&self) -> usize {
        match self.inner.exclusive_access().locked {
            true => 0,
            false => 1,
        }
    }

    fn get_allocation(&self) -> Vec<(Tid, usize)> {
        match self.inner.exclusive_access().locked {
            true => {
                let tid = self.inner.exclusive_access().allocated_to;
                vec![(tid, 1)]
            }
            false => vec![],
        }
    }

    fn get_need(&self) -> Vec<(Tid, usize)> {
        vec![]
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    allocated_to: Tid,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    allocated_to: 0,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
}

impl Resource for MutexBlocking {
    fn get_available(&self) -> usize {
        match self.inner.exclusive_access().locked {
            true => 0,
            false => 1,
        }
    }

    fn get_allocation(&self) -> Vec<(Tid, usize)> {
        let mutex_inner = self.inner.exclusive_access();
        match mutex_inner.locked {
            true => {
                let tid = mutex_inner.allocated_to;
                vec![(tid, 1)]
            }
            false => vec![],
        }
    }

    fn get_need(&self) -> Vec<(Tid, usize)> {
        let mutex_inner = self.inner.exclusive_access();
        mutex_inner
            .wait_queue
            .iter()
            .map(|tcb| (tcb.inner_exclusive_access().res.as_ref().unwrap().tid, 1))
            .collect()
    }
}

impl Mutex for MutexBlocking {
    /// lock the blocking mutex
    fn lock(&self) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
            let tid = current_task_id().unwrap();
            mutex_inner.allocated_to = tid;
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            wakeup_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
