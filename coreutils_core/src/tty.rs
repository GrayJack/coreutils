use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
};

use libc::ttyname;

use crate::file_descriptor::FileDescriptor;

use bstr::BString;

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
        let name = unsafe { ttyname(file_descriptor as i32) };

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
