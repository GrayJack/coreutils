//! Module related to system load.
use std::{io, os::raw::c_double};

use libc::getloadavg;

/// Get the 3 load average from the system.
///
/// These 3 loads represent the averages over the last 1, 5 and 15 minutes, respectively.
pub fn load_average() -> io::Result<[c_double; 3]> {
    let mut avg: [c_double; 3] = [0.0; 3];

    match unsafe { getloadavg(avg.as_mut_ptr(), 3) } {
        -1 => Err(io::Error::last_os_error()),
        _ => Ok(avg),
    }
}
