#[cfg(not(any(target_os = "fuchsia")))]
use std::ffi::CString;
use std::{convert::TryInto, io};

use crate::{
    group::{Group, Groups},
    libc,
    passwd::Passwd,
    types::Gid,
};

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

/// Set the `user` for the current process
pub fn set_user(user: Passwd) -> io::Result<()> {
    match unsafe { libc::setuid(user.uid()) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

/// Set the `groups` for the current process
pub fn set_groups(groups: Groups) -> io::Result<()> {
    let groups: Vec<Gid> = groups.iter().map(|g| g.id()).collect();
    match unsafe { libc::setgroups(groups.len().try_into().unwrap(), groups.as_ptr()) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

/// Set the `group` for the current process
pub fn set_group(group: Group) -> std::io::Result<()> {
    match unsafe { libc::setgid(group.id()) } {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}
