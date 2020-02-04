//! Module with abstractions to safely deal with processes.
#[cfg(not(any(target_os = "fuchsia")))]
use std::ffi::CString;
use std::{
    convert::TryInto,
    io::{self, Error as IOError},
};

use crate::{
    libc,
    os::{
        group::{Error as GrError, Group, Groups},
        passwd::{Error as PwError, Passwd},
        Gid,
    },
};

#[cfg(not(any(target_os = "fuchsia", target_os = "haiku")))]
pub mod priority;

/// Change the root of the running process to `newroot`.
#[cfg(not(any(target_os = "fuchsia")))]
pub fn change_root(newroot: &str) -> io::Result<()> {
    std::env::set_current_dir(newroot)?;

    let error = unsafe {
        libc::chroot(CString::new(".").unwrap().as_bytes_with_nul().as_ptr() as *const libc::c_char)
    };

    match error {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

/// Set the `user` for the current process.
pub fn set_user(user: &str) -> Result<(), PwError> {
    let user = Passwd::from_name(user)?;

    match unsafe { libc::setuid(user.uid()) } {
        0 => Ok(()),
        _ => Err(PwError::Io(IOError::last_os_error())),
    }
}

/// Set the `groups` for the current process.
pub fn set_groups(groups: &[&str]) -> Result<(), GrError> {
    let groups = Groups::from_group_list(&groups)?;
    let groups: Vec<Gid> = groups.iter().map(|g| g.id()).collect();

    match unsafe { libc::setgroups(groups.len().try_into().unwrap(), groups.as_ptr()) } {
        0 => Ok(()),
        _ => Err(GrError::Io(IOError::last_os_error())),
    }
}

/// Set the `group` for the current process.
pub fn set_group(group: &str) -> Result<(), GrError> {
    let group = Group::from_name(group)?;

    match unsafe { libc::setgid(group.id()) } {
        0 => Ok(()),
        _ => Err(GrError::Io(IOError::last_os_error())),
    }
}
