//! Module for abstractions for routing table system calls on OpenBSD.

use std::{io, os::raw::c_int};

// TODO(GrayJack): Make a pull request to expose that on libc crate
pub mod syscall {
    //! Expose publically the syscalls, use with caution.
    use std::os::raw::c_int;

    extern "C" {
        /// Returns the routing table of the current process.
        pub fn getrtable() -> c_int;

        /// Upon successful completion, setrtable() returns 0 if the call succeeds, -1 if
        /// it fails.
        pub fn setrtable(rtableid: c_int) -> c_int;
    }
}

/// Get the routing table of the current process.
#[inline]
pub fn get_routing_table() -> c_int {
    unsafe { syscall::getrtable() }
}

/// Set the routing table of `rtableid`.
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[inline]
pub fn set_routing_table(rtableid: c_int) -> io::Result<()> {
    match unsafe { syscall::setrtable(rtableid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}
