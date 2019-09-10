use libc::{gid_t, time_t, uid_t};
#[cfg(any(target_os = "freebsd", target_os = "dragonflybsd"))]
use libc::c_int;

/// Group ID type.
pub type Gid = gid_t;

/// User ID type.
pub type Uid = uid_t;

/// `Passwd` time type
pub type Time = time_t;

/// `Passwd` field type
#[cfg(any(target_os = "freebsd", target_os = "dragonflybsd"))]
pub type Fields = c_int;
