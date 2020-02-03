//! Module for safe API for getting and setting process priority

use std::{
    io::{self, Error as IOError},
    os::raw::c_int,
};

pub use libc::PRIO_PROCESS;
use libc::{getpriority, setpriority};

#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
use libc::id_t;

#[cfg(target_os = "linux")]
use libc::c_uint;

/// This function returns the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes if successful.
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn get_priority(which: c_int, who: c_int) -> io::Result<c_int> {
    let res = unsafe { getpriority(which, who) };

    match IOError::last_os_error().raw_os_error().unwrap() {
        0 => Ok(res),
        _ => Err(IOError::last_os_error()),
    }
}

/// This function returns the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes if successful.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "linux")))]
pub fn get_priority(which: c_int, who: id_t) -> io::Result<c_int> {
    let res = unsafe { getpriority(which, who) };

    match IOError::last_os_error().raw_os_error().unwrap() {
        0 => Ok(res),
        _ => Err(IOError::last_os_error()),
    }
}

/// Get the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes.
#[cfg(target_os = "linux")]
pub fn get_priority(
    #[cfg(target_env = "musl")] which: c_int, #[cfg(not(target_env = "musl"))] which: c_uint,
    who: id_t,
) -> io::Result<c_int>
{
    let res = unsafe { getpriority(which, who) };

    match IOError::last_os_error().raw_os_error().unwrap() {
        0 => Ok(res),
        _ => Err(IOError::last_os_error()),
    }
}

/// Set the priority of a specified process.
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn set_priority(which: c_int, who: c_int, prio: c_int) -> io::Result<()> {
    match unsafe { setpriority(which, who, prio) } {
        0 => Ok(()),
        _ => Err(IOError::last_os_error()),
    }
}

/// Set the priority of a specified process.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "linux")))]
pub fn set_priority(which: c_int, who: id_t, prio: c_int) -> io::Result<()> {
    match unsafe { setpriority(which, who, prio) } {
        0 => Ok(()),
        _ => Err(IOError::last_os_error()),
    }
}

/// Set the priority of a specified process.
#[cfg(target_os = "linux")]
pub fn set_priority(
    #[cfg(target_env = "musl")] which: c_int, #[cfg(not(target_env = "musl"))] which: c_uint,
    who: id_t, prio: c_int,
) -> io::Result<()>
{
    match unsafe { setpriority(which, who, prio) } {
        0 => Ok(()),
        _ => Err(IOError::last_os_error()),
    }
}
