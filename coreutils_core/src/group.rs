//! A module do deal more easily with UNIX groups.

use std::{ffi::CStr, io, ptr};

use libc::{getegid, getgrgid, getgrgid_r, getgroups, gid_t, group};
//  getgrnam_r, getgrouplist, not used for now

/// Group ID type.
pub type Gid = gid_t;

/// Contains group attributes as Rust more common types.
/// It also contains a pointer to the libc::group type for more complex manipulations.
#[derive(Clone)]
pub struct Group {
    name: String,
    id: Gid,
    passwd: String,
    gr: *mut group,
}

impl Group {
    /// Creates a new `Group` getting the user group as default.
    pub fn new() -> Self {
        let mut gr = unsafe { std::mem::zeroed() };
        let mut gr_ptr = ptr::null_mut();
        let mut buff = [0; 16384];

        unsafe {
            getgrgid_r(getegid(), &mut gr, &mut buff[0], buff.len(), &mut gr_ptr);
        }

        let name = if !gr.gr_name.is_null() {
            unsafe { CStr::from_ptr(gr.gr_name).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let id = gr.gr_gid;

        let passwd = if !gr.gr_passwd.is_null() {
            unsafe { CStr::from_ptr(gr.gr_passwd).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        Group {
            name,
            id,
            passwd,
            gr: &mut gr,
        }
    }

    /// Creates a `Group` using a `id` to get all attributes.
    pub fn new_from_gid(id: Gid) -> Self {
        let gr = unsafe { getgrgid(id) };
        let name_ptr = unsafe { (*gr).gr_name };
        let pw_name_ptr = unsafe { (*gr).gr_passwd };

        let name = if !name_ptr.is_null() {
            unsafe { CStr::from_ptr(name_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        let passwd = if !pw_name_ptr.is_null() {
            unsafe { CStr::from_ptr(pw_name_ptr).to_string_lossy().to_string() }
        } else {
            String::new()
        };

        Group {
            name,
            id,
            passwd,
            gr,
        }
    }

    /// Get the `Group` name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the `Group` id.
    pub fn id(&self) -> Gid {
        self.id
    }

    /// Get the `Group` passwd.
    pub fn passwd(&self) -> &str {
        &self.passwd
    }

    /// Get a raw pointer to the group.
    pub fn raw_ptr(&self) -> *const group {
        self.gr
    }

    // Get a mutable raw pointer to the group.
    // Use with caution.
    pub unsafe fn raw_ptr_mut(&mut self) -> *mut group {
        self.gr
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

/// Get all `Groups` that the logged user participate.
// Based of uutils get_groups
pub fn get_groups() -> io::Result<Vec<Group>> {
    let num_groups = unsafe { getgroups(0, ptr::null_mut()) };
    if num_groups == -1 {
        return Err(io::Error::last_os_error());
    }

    let mut groups_ids = Vec::with_capacity(num_groups as usize);
    let num_groups = unsafe { getgroups(num_groups, groups_ids.as_mut_ptr()) };
    if num_groups == -1 {
        return Err(io::Error::last_os_error());
    } else {
        unsafe {
            groups_ids.set_len(num_groups as usize);
        }
    }

    let groups = {
        let mut gs = Vec::with_capacity(num_groups as usize);
        for g_id in groups_ids {
            gs.push(Group::new_from_gid(g_id));
        }
        gs
    };

    Ok(groups)
}
