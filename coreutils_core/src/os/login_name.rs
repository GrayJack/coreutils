//! Module to abstract libc function that get user login name.

use std::ffi::CStr;

use bstr::BString;
// libc crate doesnt have getlogin_r, cuserid on linux target
// use libc::{getlogin, getlogin_r, cuserid};
use libc::getlogin;

/// Returns the the name of the user logged in on the controlling terminal of the process
/// if found.
#[inline]
pub fn user_login_name() -> Option<BString> {
    let res = unsafe { getlogin() };

    if res.is_null() {
        None
    } else {
        let name = unsafe { CStr::from_ptr(res) };
        Some(BString::from(name.to_bytes()))
    }
}
