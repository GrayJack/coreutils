//! Module for abstractions for routing table system calls on OpenBSD

use std::{
    error::Error as StdError,
    fmt::{self, Display},
    os::raw::c_int
}

pub mod syscall {
    //! Expose publically the syscalls, use with caution
    extern "C" {
        /// Returns the routing table of the current process
        pub fn getrtable() -> c_int;

        /// Upon successful completion, setrtable() returns 0 if the call succeeds, -1 if it fails.
        pub fn setrtable(rtableid: c_int) -> c_int;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
    err: String
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl StdError for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

/// Get the routing table of the current process
#[inline]
pub fn get_routing_table() -> c_int {
    unsafe { syscall::getrtable() }
}

/// Set the routing table of `rtableid`
pub fn set_routing_table(rtableid: c_int) -> Result<(), Error> {
    let res = unsafe { syscall::setrtable(rtableid) };

    if res < 0 {
        return Err(Error{err: "setrtable: failed to set routing table".to_string()});
    }

    Ok(())
}
