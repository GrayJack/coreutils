use super::types::TimeVal;
use std::{io, ptr};

/// Wrapper function for `libc::settimeofday`
pub fn settimeofday(timeval: TimeVal) -> io::Result<()> {
    let result = unsafe { libc::settimeofday(&timeval as *const TimeVal, ptr::null()) };
    match result {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}
