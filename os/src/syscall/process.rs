//! Process management syscalls
use crate::mm::memory_set::{_sys_mmap, _sys_munmap};
use crate::mm::page_table::translate_by_token;
use crate::syscall::{
    SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_MMAP, SYSCALL_MUNMAP, SYSCALL_SBRK, SYSCALL_TASK_INFO,
    SYSCALL_YIELD,
};
use crate::task::{count_syscall, current_user_token, task_info};
use crate::timer::{get_time_ms, get_time_us};
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
};

#[repr(C)]
#[derive(Debug)]
/// Time value in second and microsecond
pub struct TimeVal {
    /// Second
    pub sec: usize,
    /// Microsecond
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

impl TaskInfo {
    /// Create a new TaskInfo
    pub fn new() -> Self {
        Self {
            status: TaskStatus::Running,
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    count_syscall(SYSCALL_EXIT);
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    count_syscall(SYSCALL_YIELD);
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    count_syscall(SYSCALL_GET_TIME);
    trace!("kernel: sys_get_time");
    let time = get_time_us();
    let ts_mut = translate_by_token(current_user_token(), _ts);
    *ts_mut = TimeVal {
        sec: time / 1_000_000,
        usec: time % 1_000_000,
    };
    0
}

fn _sys_task_info() -> TaskInfo {
    let mut info = task_info();
    info.time = get_time_ms() - info.time;
    info
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    count_syscall(SYSCALL_TASK_INFO);
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let ti_mut = translate_by_token(current_user_token(), _ti);
    *ti_mut = _sys_task_info();
    0
}

/// YOUR JOB: Implement mmap.
/// map memory
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    count_syscall(SYSCALL_MMAP);
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    _sys_mmap(_start, _len, _port)
}

/// YOUR JOB: Implement munmap.
/// unmap memory
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    count_syscall(SYSCALL_MUNMAP);
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    _sys_munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    count_syscall(SYSCALL_SBRK);
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
