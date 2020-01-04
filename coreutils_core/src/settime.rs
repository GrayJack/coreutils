//! Temporary _settimeofday(3)_ host.
//!
//! It will probably move a `time` module or `ostime` module.

use super::types::TimeVal;
use std::{io, ptr};

/// Set the system time as `timeval`
pub fn set_time_of_day(timeval: TimeVal) -> io::Result<()> {
    let result = unsafe { libc::settimeofday(&timeval as *const TimeVal, ptr::null()) };
    match result {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}
