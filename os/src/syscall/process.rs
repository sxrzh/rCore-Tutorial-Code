//! Process management syscalls
// use crate::config::PAGE_SIZE;
use crate::mm::MapPermission;
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER};
use crate::timer::get_time_us;

use core::mem::size_of;

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
    const SIZE: usize = size_of::<TimeVal>();
    let mut data = [0u8; SIZE];
    let buf = data.as_mut_ptr() as *mut TimeVal;

    let us = get_time_us();
    unsafe {
        *buf = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }

    for i in 0..SIZE {
        if let Some(phys_addr) = TASK_MANAGER.translate_address((ts as usize + i).into(), true, true) {
            unsafe {
                let ts_real = phys_addr.0 as *mut u8;
                *ts_real = data[i];
            }
        }
        else {
            return -1;
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            if let Some(byte) = TASK_MANAGER.get_memory(id, true) {
                byte as isize
            }
            else {
                -1
            }
        }
        1 => { 
            if let Some(()) = TASK_MANAGER.set_memory(id, data as u8, true) { 0 }
            else { -1 }
        }
        2 => { TASK_MANAGER.get_syscall_count(id) as isize }
        _ => { -1 }
    }
}

/// map virtual memory
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    if prot & 0x7 == 0 { return -1; }
    if prot & !0x7 != 0 { return -1; }
    if TASK_MANAGER.map(start.into(), len, 
        MapPermission::from_bits_truncate((prot << 1) as u8) | MapPermission::U) {
            0
        }
        else {
            -1
        }
}

/// unmap virtual memory
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if TASK_MANAGER.unmap(start.into(), len) {
        0
    }
    else {
        -1
    }
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
