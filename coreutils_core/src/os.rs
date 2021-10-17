//! Module for OS processes and types abstractions.
pub mod group;
pub mod login_name;
pub mod passwd;
pub mod process;
pub mod resource;
pub mod time;
pub mod tty;
pub mod utsname;

// Specific Modules
#[cfg(not(any(target_os = "fuchsia", target_os = "haiku")))]
pub mod load;

#[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "solaris"))]
pub mod utmp;

#[cfg(not(any(target_os = "fuchsia", target_os = "openbsd")))]
pub mod utmpx;

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub mod audit;

#[cfg(target_os = "openbsd")]
pub mod routing_table;

use libc::{
    c_int, getegid, geteuid, getgid, getuid, gid_t, pid_t, suseconds_t, time_t, timeval, tm, uid_t,
};

pub type Tm = tm;

/// Time stamp type used on system structures.
pub type TimeVal = timeval;

/// Group ID type.
pub type Gid = gid_t;

/// User ID type.
pub type Uid = uid_t;

/// Process ID Type.
pub type Pid = pid_t;

/// Passwd time type.
pub type Time = time_t;

/// Passwd field type.
pub type Fields = c_int;

/// Field for [`TimeVal`] in microseconds.
pub type Susec = suseconds_t;

/// Get the current running process user effective group id.
#[inline]
pub fn get_effective_gid() -> Uid {
    unsafe { getegid() }
}

/// Get the current running process user real group id.
#[inline]
pub fn get_real_gid() -> Uid {
    unsafe { getgid() }
}

/// Get the current running process user effective user id.
#[inline]
pub fn get_effective_uid() -> Uid {
    unsafe { geteuid() }
}

/// Get the current running process user real user id.
#[inline]
pub fn get_real_uid() -> Uid {
    unsafe { getuid() }
}
