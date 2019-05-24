//! A module do deal more easily with UNIX groups.

use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    io::Error as IoError,
    ptr,
};

use libc::{getegid, getgrgid, getgrgid_r, getgrnam, getgroups, gid_t};

use bstr::{BStr, BString};

use self::GroupError::*;

/// Group ID type.
pub type Gid = gid_t;

pub type GrResult<T> = Result<T, GroupError>;

/// Enum that holds possible errors while creating `Group` type.
#[derive(Debug)]
pub enum GroupError {
    /// Happens when `getgrgid_r` or `getgrnam_r` fails.
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
    /// Happen when calling `getgroups()` C function.
    Io(IoError),
}

impl Display for GroupError {
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
        }
    }
}

impl StdError for GroupError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IoError> for GroupError {
    fn from(err: IoError) -> GroupError {
        Io(err)
    }
}

/// This struct holds information about a group of UNIX/UNIX-like systems.
///
/// Contains `sys/types.h` `group` struct attributes as Rust powefull types.
// It also contains a pointer to the libc::group type for more complex manipulations.
#[derive(Clone, Debug)]
pub struct Group {
    /// Group name.
    name: BString,
    /// Group ID.
    id: Gid,
    /// Group encrypted password
    passwd: BString,
    /// Group list of members
    mem: BString,
    // gr: *mut group
}

impl Group {
    /// Creates a new `Group` getting the user group as default.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `GroupError` wrapped in a `Err`.
    pub fn new() -> GrResult<Self> {
        let mut gr = unsafe { std::mem::zeroed() };
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384]; // Got this from manual page about `getgrgid_r`.

        let res = unsafe {
            getgrgid_r(getegid(), &mut gr, &mut buff[0], buff.len(), &mut gr_ptr)
        };

        if res != 0 {
            return Err(GetGroupFailed(String::from("getgrgid_r"), res));
        }

        if gr_ptr.is_null() {
            return Err(GroupNotFound);
        }

        let name = if !gr.gr_name.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(gr.gr_name) };
            BString::from_slice(name_cstr.to_bytes())
        } else {
            return Err(NameCheckFailed);
        };

        let id = gr.gr_gid;
        let passwd = if !gr.gr_passwd.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(gr.gr_passwd) };
            BString::from_slice(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = dbg!(unsafe { *(gr.gr_mem) });
        let mem = if !gr.gr_mem.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CStr::from_ptr(aux_ptr) };
            BString::from_slice(mem_cstr.to_bytes())
        } else {
            BString::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem
            // gr: &mut gr,
        })
    }

    /// Creates a `Group` using a `id` to get all attributes.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `GroupError` wrapped in a `Err`.
    pub fn from_gid(id: Gid) -> GrResult<Self> {
        let gr = unsafe { getgrgid(id) };
        let name_ptr = unsafe { (*gr).gr_name };
        let pw_ptr = unsafe { (*gr).gr_passwd };
        let mem_ptr = unsafe { (*gr).gr_mem };

        if gr.is_null() {
            return Err(GroupNotFound);
        }

        let name = if !name_ptr.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
            BString::from_slice(name_cstr.to_bytes())
        } else {
            return Err(NameCheckFailed);
        };

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(pw_ptr) };
            BString::from_slice(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = unsafe { *mem_ptr };
        let mem = if !mem_ptr.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CStr::from_ptr(*mem_ptr) };
            BString::from_slice(mem_cstr.to_bytes())
        } else {
            BString::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
            // gr,
        })
    }

    /// Creates a `Group` using a `name` to get all attributes.
    ///
    /// It may fail, so return a `Result`, either the `Group` struct wrapped in a `Ok`, or
    /// a `GroupError` wrapped in a `Err`.
    pub fn from_name(name: impl AsRef<[u8]>) -> GrResult<Self> {
        let name = BString::from_slice(name);

        let gr = unsafe { getgrnam((*name).as_ptr() as *const i8) };
        let pw_ptr = unsafe { (*gr).gr_passwd };
        let mem_ptr = unsafe { (*gr).gr_mem };

        if gr.is_null() {
            return Err(GroupNotFound);
        }

        let id = unsafe { (*gr).gr_gid };

        let passwd = if !pw_ptr.is_null() {
            let passwd_cstr = unsafe { CStr::from_ptr(pw_ptr) };
            BString::from_slice(passwd_cstr.to_bytes())
        } else {
            return Err(PasswdCheckFailed);
        };

        // Check if both `mem_ptr` and `*mem_ptr` are NULL since by "sys/types.h" definition
        // group.gr_mem is of type `**c_char`
        let aux_ptr = unsafe { *mem_ptr };
        let mem = if !mem_ptr.is_null() && !aux_ptr.is_null() {
            let mem_cstr = unsafe { CStr::from_ptr(*mem_ptr) };
            BString::from_slice(mem_cstr.to_bytes())
        } else {
            BString::new()
        };

        Ok(Group {
            name,
            id,
            passwd,
            mem,
            // gr,
        })
    }

    /// Get the `Group` name.
    pub fn name(&self) -> &BStr {
        &self.name
    }

    /// Get the `Group` id.
    pub fn id(&self) -> Gid {
        self.id
    }

    /// Get the `Group` encrypted password.
    pub fn passwd(&self) -> &BStr {
        &self.passwd
    }

    /// Get the `Group` list of members.
    pub fn mem(&self) -> &BStr {
        &self.mem
    }

    // /// Get a raw pointer to the group.
    // pub fn raw_ptr(&self) -> *const group {
    //     self.gr
    // }
    //
    // // Get a mutable raw pointer to the group.
    // // Use with caution.
    // pub unsafe fn raw_ptr_mut(&mut self) -> *mut group {
    //     self.gr
    // }
}

/// Get all `Groups` in the system.
///
/// It may fail, so return a `Result`, either a vector of `Group` wrapped in a `Ok`, or
/// a `GroupError` wrapped in a `Err`.
// Based of uutils get_groups
pub fn get_groups() -> GrResult<Vec<Group>> {
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

    Ok(groups)
}
