//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

use alloc::vec::Vec;
pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

type Tid = usize;

/// Resource to be shared amoung Threads of a Process
pub trait Resource {
    /// The available amount of this resource.
    fn get_available(&self) -> usize;
    /// The amounts allocated to Tids
    fn get_allocation(&self) -> Vec<(Tid, usize)>;
    /// The amount needed by Tids
    fn get_need(&self) -> Vec<(Tid, usize)>;
}
