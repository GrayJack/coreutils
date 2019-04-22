//! A module to deal more easily with UNIX passwd.

use std::{
    ffi::CStr,
    mem,
    ptr
};

use crate::group::Gid;

use libc::{passwd, uid_t, geteuid, getpwuid_r, getpwuid};

/// User ID type.
pub type Uid = uid_t;

/// Contains passwd attributes as Rust more common types.
/// It also contains a pointer to the libc::passwd type for more complex manipulations.
#[derive(Clone)]
pub struct Passwd {
    name: String,
    passwd: String,
    user_id: Uid,
    group_id: Gid,
    gecos: String,
    dir: String,
    shell: String,
    pw: *mut passwd
}

impl Passwd {
    /// Create a new `Passwd` getting the user passwd as default.
    pub fn new() -> Self {
        let mut buff = [0; 16384]; // Got this size from manual page about getpwuid_r
        let mut pw: libc::passwd = unsafe { mem::zeroed() };
        let mut pw_ptr = ptr::null_mut();

        unsafe {
            getpwuid_r(
                geteuid(),
                &mut pw,
                &mut buff[0],
                buff.len(),
                &mut pw_ptr,
            );
        }

        let name = if !pw.pw_name.is_null() {
            unsafe { CStr::from_ptr(pw.pw_name).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let passwd =  if !pw.pw_passwd.is_null() {
            unsafe { CStr::from_ptr(pw.pw_passwd).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let user_id = pw.pw_uid;

        let group_id = pw.pw_gid;

        let gecos = if !pw.pw_gecos.is_null() {
            unsafe { CStr::from_ptr(pw.pw_gecos).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let dir = if !pw.pw_dir.is_null() {
            unsafe { CStr::from_ptr(pw.pw_dir).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let shell = if !pw.pw_shell.is_null() {
            unsafe { CStr::from_ptr(pw.pw_shell).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        Passwd {
            name,
            passwd,
            user_id,
            group_id,
            gecos,
            dir,
            shell,
            pw: &mut pw
        }
    }

    /// Create a new `Passwd` using a `id` to get all attributes.
    pub fn new_from_uid(id: Uid) -> Self {
        let pw = unsafe { getpwuid(id) };
        let name_ptr = unsafe { (*pw).pw_name };
        let pw_name_ptr = unsafe { (*pw).pw_passwd };
        let gecos_ptr = unsafe { (*pw).pw_gecos };
        let dir_ptr = unsafe { (*pw).pw_dir };
        let shell_ptr = unsafe { (*pw).pw_shell };

        let name = if !name_ptr.is_null() {
            unsafe { CStr::from_ptr(name_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let passwd =  if !pw_name_ptr.is_null() {
            unsafe { CStr::from_ptr(pw_name_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let user_id = id;

        let group_id = unsafe { (*pw).pw_gid };

        let gecos = if !gecos_ptr.is_null() {
            unsafe { CStr::from_ptr(gecos_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let dir = if !dir_ptr.is_null() {
            unsafe { CStr::from_ptr(dir_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let shell = if !shell_ptr.is_null() {
            unsafe { CStr::from_ptr(shell_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        Passwd {
            name,
            passwd,
            user_id,
            group_id,
            gecos,
            dir,
            shell,
            pw
        }
    }

    /// Get `Passwd` name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get `Passwd` passwd.
    pub fn passwd(&self) -> &str {
        &self.passwd
    }

    /// Get `Passwd` user ID.
    pub fn uid(&self) -> Uid {
        self.user_id
    }

    /// Get `Passwd` group ID.
    pub fn gid(&self) -> Gid {
        self.group_id
    }

    /// Get `Passwd` gecos.
    pub fn gecos(&self) -> &str {
        &self.gecos
    }

    /// Get `Passwd` dir.
    pub fn dir(&self) -> &str {
        &self.dir
    }

    /// Get `Passwd` shell.
    pub fn shell(&self) -> &str {
        &self.shell
    }

    /// Get the raw pointer to the passwd.
    pub fn raw_ptr(&self) -> *const passwd {
        self.pw
    }

    // Get a mutable raw pointer to the passwd.
    // Use with caution.
    pub unsafe fn raw_ptr_mut(&mut self) -> *mut passwd {
        self.pw
    }
}

impl Default for Passwd {
    fn default() -> Self {
        Self::new()
    }
}
