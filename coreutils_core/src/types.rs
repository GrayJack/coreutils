//! Module for more widelly used types in this crate and helper functions related to these
//! times.
use libc::{c_int, getegid, geteuid, getgid, getuid, gid_t, pid_t, suseconds_t, time_t, timeval, uid_t};

/// Time stamp type used on system structures
pub type TimeVal = timeval;

/// Group ID type.
pub type Gid = gid_t;

/// User ID type.
pub type Uid = uid_t;

/// Process ID type
pub type Pid = pid_t;

/// `Passwd` time type
pub type Time = time_t;

/// `Passwd` field type
pub type Fields = c_int;

/// Field for `TimeStamp` in microseconds
pub type Subsec = suseconds_t;

/// Get the current running process user effective group id.
#[inline]
pub fn get_effective_gid() -> Uid { unsafe { getegid() } }

/// Get the current running process user real group id.
#[inline]
pub fn get_real_gid() -> Uid { unsafe { getgid() } }

/// Get the current running process user effective user id.
#[inline]
pub fn get_effective_uid() -> Uid { unsafe { geteuid() } }

/// Get the current running process user real user id.
#[inline]
pub fn get_real_uid() -> Uid { unsafe { getuid() } }
