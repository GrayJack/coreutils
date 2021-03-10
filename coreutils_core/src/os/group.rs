//! Module to deal more easily with UNIX groups.

#[cfg(target_os = "macos")]
use std::convert::TryInto;
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use std::os::raw::c_int;
use std::{
    convert::TryFrom,
    error::Error as StdError,
    ffi::{CStr, CString, NulError},
    fmt::{self, Display},
    io::{self, Error as IoError},
    mem::MaybeUninit,
    os::raw::c_char,
    ptr,
    slice::Iter,
};

use bstr::{BStr, BString, ByteSlice};
use libc::{getegid, getgrgid_r, getgrnam_r, getgroups, group};
#[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
use libc::{getgrouplist, getpwnam_r};
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use libc::{sysconf, _SC_NGROUPS_MAX};

use self::Error::*;
use super::{passwd::Error as PwError, Gid};

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
extern "C" {
    fn _getgroupsbymember(
        username: *const c_char, glist: *mut Gid, maxids: c_int, numgids: c_int,
    ) -> c_int;
}

/// A iterator of group members.
pub type Members = Vec<BString>;

/// Enum that holds possible errors while creating [`Group`] type.
#[derive(Debug)]
pub enum Error {
    /// Happens when [`getgrgid_r`], [`getgrnam_r`] or [`getgrouplist`] fails.
    ///
    /// It holds the the function that was used and a error code of the function return.
    GetGroupFailed(&'static str, i32),
    /// Happens when the pointer to the `.gr_name` is NULL.
    NameCheckFailed,
    /// Happens when the pointer to the `.gr_passwd` is NULL.
    PasswdCheckFailed,
    /// Happens when the pointer of [`group`] primitive is NULL.
    ///
    /// This can happen even if [`getgrgid_r`] or [`getgrnam_r`] return 0.
    GroupNotFound,
    /// Happens when creating a [`Passwd`] fails.
    Passwd(Box<PwError>),
    /// Happens when creating a [`CString`] fails.
    Cstring(NulError),
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetGroupFailed(fn_name, err_code) => write!(
                f,
                "Failed to get group with the following error code: {}. For more info search for \
                 the {} manual",
                err_code, fn_name
            ),
            NameCheckFailed => write!(f, "Group name check failed, `.gr_name` field is null"),
            PasswdCheckFailed => write!(f, "Group passwd check failed, `.gr_passwd` is null"),
            GroupNotFound => write!(f, "Group was not found in the system"),
            Passwd(err) => write!(f, "Passwd error: {}", err),
            Cstring(err) => write!(f, "Failed to create CString: {}", err),
        }
    }
}

impl StdError for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Passwd(err) => Some(err),
            Cstring(err) => Some(err),
            _ => None,
        }
    }
}

impl From<NulError> for Error {
    #[inline]
    fn from(err: NulError) -> Self {
        Cstring(err)
    }
}

impl From<PwError> for Error {
    #[inline]
    fn from(err: PwError) -> Self {
        Passwd(Box::new(err))
    }
}

impl From<Error> for IoError {
    #[inline]
    fn from(err: Error) -> Self {
        Self::new(io::ErrorKind::Other, err)
    }
}

/// This struct holds information about a group of UNIX/UNIX-like systems.
///
/// Contains `sys/types.h` [`group`] struct attributes as Rust powefull types.
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Group {
    /// Group name.
    name: BString,
    /// Group ID.
    id: Gid,
    /// Group encrypted password.
    passwd: BString,
    /// Group list of members.
    mem: Members,
}

impl Group {
    /// Creates a new [`Group`] getting the user group as default.
    ///
    /// # Errors
    /// If there is a error ocurrence when getting [`group`] (C struct) or converting it
    /// into [`Group`], an error variant is returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn new() -> io::Result<Self> {
        let mut gr = MaybeUninit::uninit();
        let mut result = ptr::null_mut();
        let buff_size = 16384; // Got this from manual page about `getgrgid_r`.
        let mut buff = Vec::with_capacity(buff_size);

        loop {
            let error_flag = unsafe {
                getgrgid_r(
                    getegid(),
                    gr.as_mut_ptr(),
                    buff.as_mut_ptr(),
                    buff.capacity(),
                    &mut result,
                )
            };

            if error_flag == 0 {
                if result.is_null() {
                    break Err(GroupNotFound.into());
                } else {
                    // Now that gr is initialized we get it
                    let gr = unsafe { gr.assume_init() };

                    break Ok(Group::try_from(gr)?);
                }
            } else if let Some(libc::ERANGE) = IoError::last_os_error().raw_os_error() {
                // If there was a ERANGE error, that means the buffer was too small, so we add more
                // `buff_size` each time we get that error
                buff.reserve(buff_size);
            } else {
                break Err(IoError::last_os_error());
            }
        }
    }

    /// Creates a [`Group`] using a `id` to get all attributes.
    ///
    /// # Errors
    /// If there is a error ocurrence when getting [`group`] (C struct) or converting it
    /// into [`Group`], an error variant is returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn from_gid(id: Gid) -> io::Result<Self> {
        let mut gr = MaybeUninit::uninit();
        let mut result = ptr::null_mut();
        let buff_size = 16384; // Got this from manual page about `getgrgid_r`.
        let mut buff = Vec::with_capacity(buff_size);

        loop {
            let error_flag = unsafe {
                getgrgid_r(id, gr.as_mut_ptr(), buff.as_mut_ptr(), buff.capacity(), &mut result)
            };

            if error_flag == 0 {
                if result.is_null() {
                    break Err(GroupNotFound.into());
                } else {
                    // Now that gr is initialized we get it
                    let gr = unsafe { gr.assume_init() };

                    break Ok(Group::try_from(gr)?);
                }
            } else if let Some(libc::ERANGE) = IoError::last_os_error().raw_os_error() {
                // If there was a ERANGE error, that means the buffer was too small, so we add more
                // `buff_size` each time we get that error
                buff.reserve(buff_size);
            } else {
                break Err(IoError::last_os_error());
            }
        }
    }

    /// Creates a [`Group`] using a `name` to get all attributes.
    ///
    /// # Errors
    /// If there is a error ocurrence when getting [`group`] (C struct) or converting it
    /// into [`Group`], an error variant is returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn from_name(name: &str) -> io::Result<Self> {
        let mut gr = MaybeUninit::uninit();
        let mut result = ptr::null_mut();
        let buff_size = 16384; // Got this from manual page about `getgrgid_r`.
        let mut buff = Vec::with_capacity(buff_size);

        let name = CString::new(name)?;

        loop {
            let error_flag = unsafe {
                getgrnam_r(
                    name.as_ptr() as *const c_char,
                    gr.as_mut_ptr(),
                    &mut buff[0],
                    buff.len(),
                    &mut result,
                )
            };

            if error_flag == 0 {
                if result.is_null() {
                    break Err(GroupNotFound.into());
                } else {
                    // Now that gr is initialized we get it
                    let gr = unsafe { gr.assume_init() };

                    break Ok(Group::try_from(gr)?);
                }
            } else if let Some(libc::ERANGE) = IoError::last_os_error().raw_os_error() {
                // If there was a ERANGE error, that means the buffer was too small, so we add more
                // `buff_size` each time we get that error
                buff.reserve(buff_size);
            } else {
                break Err(IoError::last_os_error());
            }
        }
    }

    /// Returns the `Group` name.
    #[inline]
    pub fn name(&self) -> &BStr {
        self.name.as_bstr()
    }

    /// Returns the `Group` id.
    #[inline]
    pub fn id(&self) -> Gid {
        self.id
    }

    /// Returns the `Group` encrypted password.
    #[inline]
    pub fn passwd(&self) -> &BStr {
        self.passwd.as_bstr()
    }

    /// Returns the `Group` list of members.
    #[inline]
    pub fn mem(&self) -> &Members {
        &self.mem
    }
}

impl TryFrom<group> for Group {
    type Error = Error;

    #[inline]
    fn try_from(gr: group) -> Result<Self, Self::Error> {
        let name_ptr = gr.gr_name;
        let pw_ptr = gr.gr_passwd;
        let mut mem_list_ptr = gr.gr_mem;
        let id = gr.gr_gid;

        let name = if name_ptr.is_null() {
            return Err(NameCheckFailed);
        } else {
            let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
            BString::from(name_cstr.to_bytes())
        };

        let passwd = if pw_ptr.is_null() {
            return Err(PasswdCheckFailed);
        } else {
            let passwd_cstr = unsafe { CStr::from_ptr(pw_ptr) };
            BString::from(passwd_cstr.to_bytes())
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

        Ok(Group { name, id, passwd, mem })
    }
}

/// A collection of [`Group`].
#[derive(Debug, Clone, Default)]
pub struct Groups {
    inner: Vec<Group>,
}

impl Groups {
    /// Creates a empty new [`Groups`].
    #[inline]
    pub const fn new() -> Self {
        Groups { inner: Vec::new() }
    }

    /// Get all the process caller groups.
    ///
    /// # Errors
    /// If it fails to get a [`Group`], an error variant will be returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn caller() -> io::Result<Self> {
        // First we check if we indeed have groups.
        // "If gidsetsize is 0 (fist parameter), getgroups() returns the number of supplementary
        // group IDs associated with the calling process without modifying the array
        // pointed to by grouplist."
        let num_groups = unsafe { getgroups(0, ptr::null_mut()) };
        if num_groups == -1 {
            return Err(IoError::last_os_error());
        }

        let mut groups_ids = Vec::with_capacity(num_groups as usize);
        let num_groups = unsafe { getgroups(num_groups, groups_ids.as_mut_ptr()) };
        if num_groups == -1 {
            return Err(IoError::last_os_error());
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

        Ok(Groups { inner: groups })
    }

    /// Get all groups that `username` belongs.
    ///
    /// # Errors
    /// If it fails to get a [`Group`], an error variant will be returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn from_username(username: &str) -> io::Result<Self> {
        let mut num_gr: i32 = 8;
        let mut groups_ids = Vec::with_capacity(num_gr as usize);

        let name = username.as_ptr() as *const c_char;

        let mut res = 0;
        #[cfg(not(any(target_os = "solaris", target_os = "illumos")))]
        unsafe {
            let mut passwd = MaybeUninit::uninit();
            let mut pw_result = ptr::null_mut();
            let buff_size = 16384;
            let mut buff = Vec::with_capacity(buff_size);

            let passwd = loop {
                let res_pwnam = getpwnam_r(
                    name,
                    passwd.as_mut_ptr(),
                    buff.as_mut_ptr(),
                    buff.capacity(),
                    &mut pw_result,
                );

                if res_pwnam == 0 {
                    if pw_result.is_null() {
                        return Err(Passwd(Box::new(PwError::PasswdNotFound)).into());
                    } else {
                        break passwd.assume_init();
                    }
                } else if let Some(libc::ERANGE) = IoError::last_os_error().raw_os_error() {
                    buff.reserve(buff_size);
                } else {
                    return Err(IoError::last_os_error());
                }
            };

            let gid = passwd.pw_gid;

            #[cfg(not(target_os = "macos"))]
            if getgrouplist(name, gid, groups_ids.as_mut_ptr(), &mut num_gr) == -1 {
                groups_ids.resize(num_gr as usize, 0);
                res = getgrouplist(name, gid, groups_ids.as_mut_ptr(), &mut num_gr);
            }

            #[cfg(target_os = "macos")]
            if getgrouplist(name, gid.try_into().unwrap(), groups_ids.as_mut_ptr(), &mut num_gr)
                == -1
            {
                groups_ids.resize(num_gr as usize, 0);
                res = getgrouplist(
                    name,
                    gid.try_into().unwrap(),
                    groups_ids.as_mut_ptr(),
                    &mut num_gr,
                );
            }

            groups_ids.set_len(num_gr as usize);
        }
        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        unsafe {
            if _getgroupsbymember(name, groups_ids.as_mut_ptr(), num_gr, 0) == -1 {
                // Fist we tried with the pre-defined one, now we get the true max
                num_gr = sysconf(_SC_NGROUPS_MAX) as c_int;
                groups_ids.resize(num_gr as usize, 0);
                res = _getgroupsbymember(name, groups_ids.as_mut_ptr(), num_gr, 0);
            }
            groups_ids.set_len(num_gr as usize);
        }

        if res == -1 && (cfg!(target_os = "solaris") || cfg!(target_os = "illumos")) {
            return Err(GetGroupFailed("_getgroupsbymember", res).into());
        } else if res == -1 {
            return Err(GetGroupFailed("getgrouplist", res).into());
        }

        groups_ids.truncate(num_gr as usize);

        let groups = {
            let mut gs = Vec::with_capacity(num_gr as usize);
            for gid in groups_ids {
                #[cfg(not(target_os = "macos"))]
                let gr = Group::from_gid(gid)?;
                #[cfg(target_os = "macos")]
                let gr = Group::from_gid(gid as u32)?;

                gs.push(gr);
            }
            gs
        };

        Ok(Groups { inner: groups })
    }

    /// Get groups from a list of group names.
    ///
    /// # Errors
    /// If it fails to get a [`Group`], an error variant will be returned.
    #[inline]
    pub fn from_group_list(group_list: &[&str]) -> io::Result<Self> {
        let groups: io::Result<Vec<Group>> =
            group_list.iter().map(|&group_name| Group::from_name(group_name)).collect();

        groups.map(|groups| Self { inner: groups })
    }

    /// Insert a [`Group`] on [`Groups`].
    #[inline]
    pub fn push(&mut self, value: Group) {
        self.inner.push(value);
    }

    /// Return `true` if [`Groups`] contains 0 elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Transform [`Groups`] to a Vector of [`Group`].
    #[inline]
    pub fn into_vec(self) -> Vec<Group> {
        self.inner
    }

    /// Creates a iterator over it's entries.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Group> {
        self.inner.iter()
    }
}

impl IntoIterator for Groups {
    type IntoIter = std::vec::IntoIter<Group>;
    type Item = Group;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}


// Extra traits
impl From<Group> for group {
    #[inline]
    fn from(mut gr: Group) -> Self {
        let mut vec: Vec<*mut c_char> =
            gr.mem.iter().map(|s| s.clone().as_mut_ptr() as *mut c_char).collect();

        group {
            gr_name: gr.name.as_mut_ptr() as *mut c_char,
            gr_gid: gr.id,
            gr_passwd: gr.passwd.as_mut_ptr() as *mut c_char,
            gr_mem: vec.as_mut_ptr(),
        }
    }
}
