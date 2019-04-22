use std::{ffi::CStr, io, ptr};

use libc::{getgrgid, getgrgid_r, getgrnam_r, getgrouplist, getgroups, gid_t, group};

pub type Gid = gid_t;

#[derive(Clone)]
pub struct Group {
    id: Gid,
    name: String,
    passwd_name: String,
    gr: *const group,
}

impl Group {
    pub fn new_from_gid(id: Gid) -> Self {
        let gr = unsafe { getgrgid(id) };
        let name_ptr = unsafe { (*gr).gr_name };
        let pw_name_ptr = unsafe { (*gr).gr_passwd };

        let name = if !name_ptr.is_null() {
            unsafe { CStr::from_ptr(name_ptr).to_string_lossy().to_string() }
        } else {
            "".to_string()
        };

        let passwd_name = if !pw_name_ptr.is_null() {
            unsafe { CStr::from_ptr(pw_name_ptr).to_string_lossy().to_string() }
        } else {
            "".to_string()
        };

        Group {
            id,
            name,
            passwd_name,
            gr,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> Gid {
        self.id
    }

    pub fn passwd_name(&self) -> &str {
        &self.passwd_name
    }

    pub unsafe fn raw_ptr(&self) -> *const group {
        self.gr
    }
}

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

fn group_name(id: gid_t) -> Option<String> {
    let gr = unsafe { getgrgid(id) };
    let group_name = unsafe { (*gr).gr_name };

    if group_name.is_null() {
        return None;
    }

    let group_name = unsafe { CStr::from_ptr(group_name).to_string_lossy().to_string() };
    Some(group_name)
}

//
// let mut gr: Group = unsafe { std::mem::zeroed() };
// let mut gr_ptr = ptr::null_mut();
// let mut buff = [0; 16384];
//
// unsafe {
//     getgrgid_r(id, &mut gr.g, &mut buff[0], buff.len(), &mut gr_ptr);
// }
//
// gr.g.gr_name = {
//     let gr = unsafe { getgrgid(id) };
//     unsafe{ (*gr).gr_name }
// };
// gr
