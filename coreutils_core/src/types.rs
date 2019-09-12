#[cfg(any(target_os = "freebsd", target_os = "dragonflybsd"))]
use libc::c_int;
use libc::{getegid, geteuid, getgid, getuid, gid_t, time_t, uid_t};

/// Group ID type.
pub type Gid = gid_t;

/// Get the current running process user effective group id.
pub fn get_effective_gid() -> Uid {
    unsafe { getegid() }
}

/// Get the current running process user real group id.
pub fn get_real_gid() -> Uid {
    unsafe { getgid() }
}

/// User ID type.
pub type Uid = uid_t;

/// Get the current running process user effective user id.
pub fn get_effective_uid() -> Uid {
    unsafe { geteuid() }
}

/// Get the current running process user real user id.
pub fn get_real_uid() -> Uid {
    unsafe { getuid() }
}

/// `Passwd` time type
pub type Time = time_t;

/// `Passwd` field type
#[cfg(any(target_os = "freebsd", target_os = "dragonflybsd"))]
pub type Fields = c_int;
