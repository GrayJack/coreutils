//! Module for safe API for

use std::{
    error::Error as StdError,
    fmt::{self, Display},
    io::Error as IOError,
    os::raw::c_int,
};

pub use libc::PRIO_PROCESS;
use libc::{getpriority, setpriority};

#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly")))]
use libc::id_t;

#[cfg(target_os = "linux")]
use libc::c_uint;

/// Possible errors
#[derive(Debug)]
pub enum Error {
    SetPriority(IOError),
    GetPriority(IOError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SetPriority(ioerr) => {
                write!(f, "setpriority: failed to set priority: {}", ioerr)
            },
            Self::GetPriority(ioerr) => {
                write!(f, "getpriority: failed to get priority: {}", ioerr)
            },
        }
    }
}

impl StdError for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::SetPriority(ioerr) => Some(ioerr),
            Self::GetPriority(ioerr) => Some(ioerr),
        }
    }
}

/// This function returns the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes if successful.
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn get_priority(which: c_int, who: c_int) -> Result<c_int, Error> {
    let res = unsafe { getpriority(which, who) };

    if IOError::last_os_error().raw_os_error().unwrap() != 0 {
        return Err(Error::GetPriority(IOError::last_os_error()));
    }

    Ok(res)
}

/// This function returns the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes if successful.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "linux")))]
pub fn get_priority(which: c_int, who: id_t) -> Result<c_int, Error> {
    let res = unsafe { getpriority(which, who) };

    if IOError::last_os_error().raw_os_error().unwrap() != 0 {
        return Err(Error::GetPriority(IOError::last_os_error()));
    }

    Ok(res)
}

/// Get the highest priority (lowest numerical value) enjoyed by any of
/// the specified processes.
#[cfg(target_os = "linux")]
pub fn get_priority(which: c_uint, who: id_t) -> Result<c_int, Error> {
    #[cfg(target_env = "musl")]
    let res = unsafe { getpriority(which as c_int, who) };

    #[cfg(not(target_env = "musl"))]
    let res = unsafe { getpriority(which, who) };

    if IOError::last_os_error().raw_os_error().unwrap() != 0 {
        return Err(Error::GetPriority(IOError::last_os_error()));
    }

    Ok(res)
}

/// Set the priority of a specified process.
#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
pub fn set_priority(which: c_int, who: c_int, prio: c_int) -> Result<(), Error> {
    let res = unsafe { setpriority(which, who, prio) };

    if res < 0 {
        return Err(Error::SetPriority(IOError::last_os_error()));
    }

    Ok(())
}

/// Set the priority of a specified process.
#[cfg(not(any(target_os = "freebsd", target_os = "dragonfly", target_os = "linux")))]
pub fn set_priority(which: c_int, who: id_t, prio: c_int) -> Result<(), Error> {
    let res = unsafe { setpriority(which, who, prio) };

    if res < 0 {
        return Err(Error::SetPriority(IOError::last_os_error()));
    }

    Ok(())
}

/// Set the priority of a specified process.
#[cfg(target_os = "linux")]
pub fn set_priority(which: c_uint, who: id_t, prio: c_int) -> Result<(), Error> {
    #[cfg(target_env = "musl")]
    let res = unsafe { setpriority(which as c_int, who, prio) };

    #[cfg(not(target_env = "musl"))]
    let res = unsafe { setpriority(which, who, prio) };

    if res < 0 {
        return Err(Error::SetPriority(IOError::last_os_error()));
    }

    Ok(())
}
