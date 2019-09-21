//! Module for abstractions for routing table system calls on OpenBSD

use std::os::raw::c_int

mod syscall {
    extern "C" {
        /// Returns the routing table of the current process
        pub fn getrtable() -> c_int;

        /// Upon successful completion, setrtable() returns 0 if the call succeeds, -1 if it fails.
        pub fn setrtable(rtableid: c_int) -> c_int;
    }
}

/// Get the routing table of the current process
pub fn get_routing_table() -> c_int {
    unsafe { getrtable() }
}
