//! Process management syscalls

use crate::{
    config::MAX_SYSCALL_NUM,
    mm::translated_byte_buffer,
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_info,
        memory_alloc, suspend_current_and_run_next, TaskControlBlock, TaskStatus,memory_free
    },
    timer::{get_time_ms, get_time_us},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

impl From<&TaskControlBlock> for TaskInfo {
    fn from(tcb: &TaskControlBlock) -> Self {
        TaskInfo {
            status: tcb.task_status,
            syscall_times: tcb.syscall_times,
            time: get_time_ms() - tcb.time.unwrap_or(0),
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let buffers = translated_byte_buffer(
        current_user_token(),
        ts as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    let us = get_time_us();
    let mut timeval_ptr = (&TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    }) as *const _ as *const u8;
    for buffer in buffers {
        unsafe {
            timeval_ptr.copy_to(buffer.as_mut_ptr(), buffer.len());
            timeval_ptr = timeval_ptr.add(buffer.len());
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let buffers = translated_byte_buffer(
        current_user_token(),
        ti as *const u8,
        core::mem::size_of::<TaskInfo>(),
    );
    let mut info_ptr = (&get_current_task_info::<TaskInfo>()) as *const _ as *const u8;
    for buffer in buffers {
        unsafe {
            info_ptr.copy_to(buffer.as_mut_ptr(), buffer.len());
            info_ptr = info_ptr.add(buffer.len());
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if memory_alloc(start, start + len, port) {
        return 0;
    }
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if memory_free(start, start + len) {
        return 0;
    }
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
