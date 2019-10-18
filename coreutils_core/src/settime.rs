use libc::timeval;
use std::ptr;
use errno::errno;

/// Wrapper function for `libc::settimeofday`
pub fn settimeofday(tv_sec: i64, tv_nsec: i32) -> Result<(), String> {
    let timeval = timeval {
        tv_sec,
        tv_usec: tv_nsec
    };

    unsafe {
        let result = libc::settimeofday(&timeval as *const timeval, ptr::null());
        match result {
            0 => Ok(()),
            _ => {
                Err(errno().to_string())
            }
        }
    }
}