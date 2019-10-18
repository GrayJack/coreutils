use std::{ptr, io};
use super::types::TimeVal;

/// Wrapper function for `libc::settimeofday`
pub fn settimeofday(timeval: TimeVal) -> io::Result<()> {
    let result = unsafe { libc::settimeofday(&timeval as *const TimeVal, ptr::null()) };
    match result {
        0 => Ok(()),
        _ => {
            Err(io::Error::last_os_error())
        }
    }
}