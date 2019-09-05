use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    ptr,
};

use libc::ttyname_r;

use crate::file_descriptor::FileDescriptor;

use bstr::BString;

mod consts {
    pub const TTY_NAME_MAX: usize = 32;
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    NotTTY,
    LibcCall(String, i32),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotTTY => write!(f, "Not a TTY"),
            Self::LibcCall(fn_name, err_code) => write!(
                f,
                "Failed calling {} with this error code: {}",
                fn_name, err_code
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TTYName(BString);

impl TTYName {
    pub fn new(file_descriptor: FileDescriptor) -> Result<Self, Error> {
        let name = ptr::null_mut();
        let size = consts::TTY_NAME_MAX; // This way we ensure that ERANGE will not happen

        let res = unsafe { ttyname_r(file_descriptor as i32, name, size) };

        if res != 0 {
            if res == libc::ENOTTY {
                return Err(Error::NotTTY);
            }

            return Err(Error::LibcCall(String::from("ttyname_r"), res));
        }

        let name = if !name.is_null() {
            let name_cstr = unsafe { CStr::from_ptr(name) };
            BString::from(name_cstr.to_bytes())
        } else {
            return Err(Error::NotTTY);
        };

        Ok(TTYName(name))
    }
}

impl Display for TTYName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn isatty(file_descriptor: FileDescriptor) -> bool {
    unsafe { libc::isatty(file_descriptor as i32) == 1 }
}
