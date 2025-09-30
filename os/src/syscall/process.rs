//! Process management syscalls

use core::{mem::size_of, slice::from_raw_parts};

use crate::{
    mm::{read_a_byte, translated_byte_buffer, write_a_byte},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        query_current_syscall_stat, suspend_current_and_run_next, mmap_for_current, munmap_for_current
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
    let us = get_time_us();
    let buffers =
        translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let ts_tmp = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let ts_tmp_byte_ptr = &ts_tmp as *const TimeVal as *const u8;
    let ts_tmp_slice = unsafe { from_raw_parts(ts_tmp_byte_ptr, size_of::<TimeVal>()) };
    let mut pos = 0;
    for buffer in buffers {
        buffer.copy_from_slice(&ts_tmp_slice[pos..buffer.len()]);
        pos += buffer.len();
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    // read a byte from id (as address)
    const SYS_TRACE_READ: usize = 0;
    // write a byte to id (as address)
    const SYS_TRACE_WRITE: usize = 1;
    // query number of syscall of id called by current task
    const SYS_TRACE_QUERY: usize = 2;

    match trace_request {
        SYS_TRACE_READ => read_a_byte(current_user_token(), id as *const u8),
        SYS_TRACE_WRITE => write_a_byte(current_user_token(), id as *mut u8, data as u8),
        SYS_TRACE_QUERY => query_current_syscall_stat(id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    mmap_for_current(start, len, prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    munmap_for_current(start, len)
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
