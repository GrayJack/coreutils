//! Module to deal more easily with UNIX groups.

use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    io::Error as IoError,
    mem::MaybeUninit,
    os::raw::c_char,
    ptr,
};

#[cfg(target_os = "macos")]
use std::convert::TryInto;

use libc::{getegid, getgrgid_r, getgrnam_r, getgrouplist, getgroups, getpwnam};

use bstr::{BStr, BString, ByteSlice};

use self::Error::*;
use crate::{types::Gid, passwd::Error as PwError};

pub type Result<T> = std::result::Result<T, Error>;

/// A iterator of group members.
pub type Members = Vec<BString>;

/// Enum that holds possible errors while creating `Group` type.
#[derive(Debug)]
pub enum Error {
    /// Happens when `getgrgid_r`, `getgrnam_r` or `getgrouplist` fails.
    ///
    /// It holds the the function that was used and a error code of the function return.
    GetGroupFailed(String, i32),
    /// Happens when the pointer to the `.gr_name` is NULL.
    NameCheckFailed,
    /// Happens when the pointer to the `.gr_passwd` is NULL.
    PasswdCheckFailed,
    /// Happens when the pointer of `group` primitive is NULL.
    ///
    /// This can happen even if `getgrgid_r` or `getgrnam_r` return 0.
    GroupNotFound,
    /// Happens when calling `getgroups()` or `getgrouplist()` C function.
    Io(IoError),
    /// Happens when creating a `Passwd` fails.
    Passwd(Box<PwError>),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetGroupFailed(fn_name, err_code) => write!(
                f,
                "Failed to get group with the following error code: {}. For more info search for the {} manual",
                err_code, fn_name
            ),
            NameCheckFailed => write!(f, "Group name check failed, `.gr_name` field is null"),
            PasswdCheckFailed => write!(f, "Group passwd check failed, `.gr_passwd` is null"),
            GroupNotFound => write!(f, "Group was not found in the system"),
            Io(err) => write!(f, "The following error hapenned trying to get all `Groups`: {}", err),
            Passwd(err) => write!(f, "The following error hapenned trying to get all `Groups`: {}", err),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Io(err) => Some(err),
            Passwd(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Io(err)
    }
}

impl From<PwError> for Error {
    fn from(err: PwError) -> Error {
        Passwd(Box::new(err))
    }
}

/// This struct holds information about a group of UNIX/UNIX-like systems.
///
/// Contains `sys/types.h` `group` struct attributes as Rust powefull types.
// It also contains a pointer to the libc::group type for more complex manipulations.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Group {
    /// Group name.
    name: BString,
    /// Group ID.
    id: Gid,
    /// Group encrypted password
    passwd: BString,
    /// Group list of members
    mem: Members,
}

impl Group {
    /// Creates a new `Group` getting the user group as default.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `Error` wrapped in a `Err`.
    pub fn new() -> Result<Self> {
        let mut gr = MaybeUninit::zeroed();
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384]; // Got this from manual page about `getgrgid_r`.

        let res = unsafe {
            getgrgid_r(
                getegid(),
                gr.as_mut_ptr(),
                &mut buff[0],
                buff.len(),
                &mut gr_ptr,
            )
        };

        if gr_ptr.is_null() {
            if res == 0 {
                return Err(GroupNotFound);
            } else {
                return Err(GetGroupFailed(String::from("getgrgid_r"), res));
            }
        }

        // Now that pw is initialized we get it
        let gr = unsafe { gr.assume_init() };

        let name = if !gr.gr_name.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(gr.gr_name) };
            BString::from(name_cstr.to_bytes())
        } else {
            return Err(NameCheckFailed);
        };

        let id = gr.gr_gid;
        let passwd = if !gr.gr_passwd.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(gr.gr_passwd) };
            BString::from(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let mut mem_list_ptr = gr.gr_mem;
        let mut mem_ptr = unsafe { *mem_list_ptr };
        let mem = if !mem_list_ptr.is_null() && !mem_ptr.is_null() {
            let mut members: Members = Members::new();
            while !mem_list_ptr.is_null() && !mem_ptr.is_null() {
                let mem_cstr = unsafe { CStr::from_ptr(mem_ptr) };
                members.push(BString::from(mem_cstr.to_bytes()));
                mem_list_ptr = unsafe { mem_list_ptr.add(1) };
                mem_ptr = unsafe { *mem_list_ptr };
            }
            members
        } else {
            Members::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
        })
    }

    /// Creates a `Group` using a `id` to get all attributes.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `Error` wrapped in a `Err`.
    pub fn from_gid(id: Gid) -> Result<Self> {
        let mut gr = MaybeUninit::zeroed();
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384]; // Got this from manual page about `getgrgid_r`.

        let res = unsafe { getgrgid_r(id, gr.as_mut_ptr(), &mut buff[0], buff.len(), &mut gr_ptr) };

        if gr_ptr.is_null() {
            if res == 0 {
                return Err(GroupNotFound);
            } else {
                return Err(GetGroupFailed(String::from("getgrgid_r"), res));
            }
        }

        // Now that pw is initialized we get it
        let gr = unsafe { gr.assume_init() };

        let name_ptr = gr.gr_name;
        let pw_ptr = gr.gr_passwd;
        let mut mem_list_ptr = gr.gr_mem;

        let name = if !name_ptr.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
            BString::from(name_cstr.to_bytes())
        } else {
            return Err(NameCheckFailed);
        };

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(pw_ptr) };
            BString::from(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let mut mem_ptr = unsafe { *mem_list_ptr };
        let mem = if !mem_list_ptr.is_null() && !mem_ptr.is_null() {
            let mut members: Members = Members::new();

            while !mem_list_ptr.is_null() && !mem_ptr.is_null() {
                let mem_cstr = unsafe { CStr::from_ptr(mem_ptr) };
                members.push(BString::from(mem_cstr.to_bytes()));

                // Update pointers
                mem_list_ptr = unsafe { mem_list_ptr.add(1) };
                mem_ptr = unsafe { *mem_list_ptr };
            }
            members
        } else {
            Members::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
        })
    }

    /// Creates a `Group` using a `name` to get all attributes.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `Error` wrapped in a `Err`.
    pub fn from_name(name: &str) -> Result<Self> {
        let mut gr = MaybeUninit::zeroed();
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384]; // Got this from manual page about `getgrgid_r`.

        let name = BString::from(name);

        let res = unsafe {
            getgrnam_r(
                name.as_ptr() as *const c_char,
                gr.as_mut_ptr(),
                &mut buff[0],
                buff.len(),
                &mut gr_ptr,
            )
        };

        if gr_ptr.is_null() {
            if res == 0 {
                return Err(GroupNotFound);
            } else {
                return Err(GetGroupFailed(String::from("getgrgid_r"), res));
            }
        }

        // Now that pw is initialized we get it
        let gr = unsafe { gr.assume_init() };

        let pw_ptr = gr.gr_passwd;
        let mut mem_list_ptr = gr.gr_mem;

        let id = gr.gr_gid;

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(pw_ptr) };
            BString::from(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let mut mem_ptr = unsafe { *mem_list_ptr };
        let mem = if !mem_list_ptr.is_null() && !mem_ptr.is_null() {
            let mut members: Members = Members::new();

            while !mem_list_ptr.is_null() && !mem_ptr.is_null() {
                let mem_cstr = unsafe { CStr::from_ptr(mem_ptr) };
                members.push(BString::from(mem_cstr.to_bytes()));

                // Update pointers
                mem_list_ptr = unsafe { mem_list_ptr.add(1) };
                mem_ptr = unsafe { *mem_list_ptr };
            }
            members
        } else {
            Members::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
        })
    }

    /// Get the `Group` name.
    pub fn name(&self) -> &BStr {
        &self.name.as_bstr()
    }

    /// Get the `Group` id.
    pub fn id(&self) -> Gid {
        self.id
    }

    /// Get the `Group` encrypted password.
    pub fn passwd(&self) -> &BStr {
        &self.passwd.as_bstr()
    }

    /// Get the `Group` list of members.
    pub fn mem(&self) -> &Members {
        &self.mem
    }
}

/// A collection of `Group`.
#[derive(Debug, Clone, Default)]
pub struct Groups {
    iter: Vec<Group>,
}

impl Groups {
    /// Creates a empty new `Groups`.
    pub fn new() -> Self {
        Groups { iter: Vec::new() }
    }

    /// Get all the process caller groups
    pub fn caller() -> Result<Self> {
        // First we check if we indeed have groups.
        // "If gidsetsize is 0 (fist parameter), getgroups() returns the number of supplementary group
        // IDs associated with the calling process without modifying the array pointed to by grouplist."
        let num_groups = unsafe { getgroups(0, ptr::null_mut()) };
        if num_groups == -1 {
            return Err(Io(IoError::last_os_error()));
        }

        let mut groups_ids = Vec::with_capacity(num_groups as usize);
        let num_groups = unsafe { getgroups(num_groups, groups_ids.as_mut_ptr()) };
        if num_groups == -1 {
            return Err(Io(IoError::last_os_error()));
        } else {
            unsafe {
                groups_ids.set_len(num_groups as usize);
            }
        }

        let groups = {
            let mut gs = Vec::with_capacity(num_groups as usize);
            for g_id in groups_ids {
                if let Ok(gr) = Group::from_gid(g_id) {
                    gs.push(gr);
                }
            }
            gs
        };

        Ok(Groups { iter: groups })
    }

    /// Get all groups that `username` belongs.
    #[cfg(not(any(target_os = "macos")))]
    pub fn from_username(username: &str) -> Result<Self> {
        let mut num_gr: i32 = 8;
        let mut groups_ids = Vec::with_capacity(num_gr as usize);

        let passwd = unsafe { getpwnam(username.as_ptr() as *const c_char) };

        let name = username.as_ptr() as *const c_char;
        let gid = unsafe { (*passwd).pw_gid };

        let mut res = 0;
        unsafe {
            if getgrouplist(name, gid, groups_ids.as_mut_ptr(), &mut num_gr) == -1 {
                groups_ids.resize(num_gr as usize, 0);
                res = getgrouplist(name, gid, groups_ids.as_mut_ptr(), &mut num_gr);
            }
            groups_ids.set_len(num_gr as usize);
        }

        if res == -1 {
            return Err(GetGroupFailed(String::from("getgrouplist"), res));
        }

        groups_ids.truncate(num_gr as usize);

        let groups = {
            let mut gs = Vec::with_capacity(num_gr as usize);
            for gid in groups_ids {
                let gr = Group::from_gid(gid)?;
                gs.push(gr);
            }
            gs
        };

        Ok(Groups { iter: groups })
    }

    /// Get all groups that `username` belongs.
    #[cfg(any(target_os = "macos"))]
    pub fn from_username(username: &str) -> Result<Self> {
        let mut num_gr: i32 = 8;
        let mut groups_ids = Vec::with_capacity(num_gr as usize);

        let passwd = unsafe { getpwnam(username.as_ptr() as *const c_char) };

        let name = username.as_ptr() as *const c_char;
        let gid = unsafe { (*passwd).pw_gid };

        let mut res = 0;
        unsafe {
            if getgrouplist(name, gid.try_into().unwrap(), groups_ids.as_mut_ptr(), &mut num_gr) == -1 {
                groups_ids.resize(num_gr as usize, 0);
                res = getgrouplist(name, gid.try_into().unwrap(), groups_ids.as_mut_ptr(), &mut num_gr);
            }
            groups_ids.set_len(num_gr as usize);
        }

        if res == -1 {
            return Err(GetGroupFailed(String::from("getgrouplist"), res));
        }

        groups_ids.truncate(num_gr as usize);

        let groups = {
            let mut gs = Vec::with_capacity(num_gr as usize);
            for gid in groups_ids {
                let gr = Group::from_gid(gid.try_into().unwrap())?;
                gs.push(gr);
            }
            gs
        };

        Ok(Groups { iter: groups })
    }

    /// Insert as `Group` on `Groups`.
    pub fn push(&mut self, value: Group) {
        self.iter.push(value);
    }

    /// Return `true` if `Groups` contains 0 elements.
    pub fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }

    /// Transform `Groups` to a Vector of `Group`.
    pub fn into_vec(self) -> Vec<Group> {
        self.iter
    }
}

impl IntoIterator for Groups {
    type Item = Group;
    type IntoIter = std::vec::IntoIter<Group>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter.into_iter()
    }
}
