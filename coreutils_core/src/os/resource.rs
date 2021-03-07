//! Module abstracting interactions with getrusage(2)
//!
//! Also holds utility functions for summarizing the data returned by getrusage(2)
use super::TimeVal;
#[cfg(not(target_os = "fuchsia"))]
use libc::getrusage;
use libc::{c_int, rusage, RUSAGE_CHILDREN, RUSAGE_SELF};

use std::convert::From;

/// Interface for `RUSAGE_*` constants from libc.
///
/// TODO This is an incomplete set of constants. It is currently missing
/// `libc::RUSAGE_THREAD` which requires the `_GNU_SOURCE` macro to be defined
/// at build time.
pub enum ResourceConsumer {
    Caller = RUSAGE_SELF as isize,
    Children = RUSAGE_CHILDREN as isize,
}

#[derive(Debug)]
pub struct RUsage {
    pub timing: Timing,
    pub mem:    MemoryUsage,
    pub io:     IOUsage,
}

#[derive(Debug)]
pub struct Timing {
    /// User CPU time used
    pub user_time: TimeVal,
    /// System CPU time used
    pub sys_time:  TimeVal,
}

#[derive(Debug)]
pub struct MemoryUsage {
    /// Maximum resident set size
    pub max_rss: u64,
    /// Number of page reclaims (soft page faults)
    pub num_minor_page_flt: u64,
    /// Number of page faults (hard page faults)
    pub num_major_page_flt: u64,
    /// Number of voluntary context switches
    pub num_vol_ctx_switch: u64,
    /// Number of involuntary context switches
    pub num_invol_ctx_switch: u64,
    /// Unmaintained on linux: Integral shared memory size
    pub shared_mem_size: u64,
    /// Unmaintained on linux: Integral unshared data size
    pub unshared_data_size: u64,
    /// Unmaintained on linux: Integral unshared stack size
    pub unshared_stack_size: u64,
    /// Unmaintained on linux: Number of swaps
    pub num_swaps: u64,
}

#[derive(Debug)]
pub struct IOUsage {
    /// Number of block input operations
    pub num_block_in:  u64,
    /// Number of block output operations
    pub num_block_out: u64,
    /// Unmaintained on linux: Number of IPC messages recieved
    pub num_sock_recv: u64,
    /// Unmaintained on linux: Number of IPC messages sent
    pub num_sock_send: u64,
    /// Unmaintained: Number of signals recieved
    pub num_signals:   u64,
}

impl From<rusage> for RUsage {
    fn from(ru: rusage) -> Self {
        RUsage {
            timing: Timing { user_time: ru.ru_utime, sys_time: ru.ru_stime },
            mem:    MemoryUsage {
                max_rss: ru.ru_maxrss as u64,
                num_minor_page_flt: ru.ru_minflt as u64,
                num_major_page_flt: ru.ru_majflt as u64,
                num_vol_ctx_switch: ru.ru_nvcsw as u64,
                num_invol_ctx_switch: ru.ru_nivcsw as u64,
                shared_mem_size: ru.ru_ixrss as u64,
                unshared_data_size: ru.ru_idrss as u64,
                unshared_stack_size: ru.ru_isrss as u64,
                num_swaps: ru.ru_nswap as u64,
            },
            io:     IOUsage {
                num_block_in:  ru.ru_inblock as u64,
                num_block_out: ru.ru_oublock as u64,
                num_sock_recv: ru.ru_msgrcv as u64,
                num_sock_send: ru.ru_msgsnd as u64,
                num_signals:   ru.ru_nsignals as u64,
            },
        }
    }
}

/// Safely wrap `libc::getrusage`
pub fn get_rusage(target: ResourceConsumer) -> RUsage {
    let mut usage: rusage = unsafe { std::mem::zeroed() };

    #[cfg(not(target_os = "fuchsia"))]
    // Fuchsia doesn't have a getrusage syscall, but provides the rusage struct.
    // The default is to abort with an error message so that callers don't end
    // up with invalid data.
    unsafe {
        getrusage(target as c_int, &mut usage);
    }

    RUsage::from(usage)
}
