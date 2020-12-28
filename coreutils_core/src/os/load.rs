//! Module related to system load.
use std::os::raw::c_double;

use libc::getloadavg;

/// Returns 3 load averages from the system.
///
/// These 3 loads represent the averages over the last 1, 5 and 15 minutes, respectively.
#[inline]
pub fn load_average() -> Option<[c_double; 3]> {
    let mut avg: [c_double; 3] = [0.0; 3];

    match unsafe { getloadavg(avg.as_mut_ptr(), 3) } {
        -1 => None,
        _ => Some(avg),
    }
}
