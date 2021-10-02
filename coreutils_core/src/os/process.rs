//! Module with abstractions to safely deal with processes.
#[cfg(not(any(target_os = "fuchsia")))]
use std::ffi::CString;
use std::{convert::TryInto, io};

use crate::{
    libc,
    os::{
        group::{Group, Groups},
        passwd::Passwd,
        Gid,
    },
};

#[cfg(not(any(target_os = "fuchsia")))]
pub mod priority;

/// Change the root of the running process to `newroot`.
///
/// # Errors
/// If a internal call set a errno (I/O OS error), an error variant will be returned.
#[cfg(not(any(target_os = "fuchsia")))]
#[inline]
pub fn change_root(newroot: &str) -> io::Result<()> {
    std::env::set_current_dir(newroot)?;

    let error = unsafe {
        libc::chroot(CString::new(".").unwrap().as_bytes_with_nul().as_ptr() as *const libc::c_char)
    };

    match error {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

/// Set the `user` for the current process.
///
/// # Errors
/// If a internal call set a errno (I/O OS error) or it fails to get [`Passwd`], an error
/// variant will be returned.
///
/// [`Passwd`]: ../passwd/struct.Passwd.html
#[inline]
pub fn set_user(user: &str) -> io::Result<()> {
    let user = Passwd::from_name(user)?;

    match unsafe { libc::setuid(user.uid()) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

/// Set the `groups` for the current process.
///
/// # Errors
/// If a internal call set a errno (I/O OS error) or it fails to get [`Groups`], an error
/// variant will be returned.
#[inline]
pub fn set_groups(groups: &[&str]) -> io::Result<()> {
    let groups = Groups::from_group_list(groups)?;
    let groups: Vec<Gid> = groups.iter().map(|g| g.id()).collect();

    #[allow(clippy::useless_conversion)]
    let size = groups.len().try_into().unwrap_or_default();

    match unsafe { libc::setgroups(size, groups.as_ptr()) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

/// Set the `group` for the current process.
///
/// # Errors
/// If a internal call set a errno (I/O OS error) or it fails to get [`Group`], an error
/// variant will be returned.
#[inline]
pub fn set_group(group: &str) -> io::Result<()> {
    let group = Group::from_name(group)?;

    match unsafe { libc::setgid(group.id()) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}
