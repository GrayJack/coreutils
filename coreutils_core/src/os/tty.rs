//! Module for TTY abstractions.

use std::{
    error::Error as StdError,
    ffi::CStr,
    fmt::{self, Display},
    os::raw::c_int,
};

use libc::ttyname;

// use crate::file_descriptor::FileDescriptor;

use bstr::{BStr, BString, ByteSlice};

/// Possible errors while trying to get a TTY name
#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    /// Not a TTY error
    NotTTY,
    /// Any other error
    LibcCall(String, i32),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotTTY => write!(f, "Not a TTY"),
            Self::LibcCall(fn_name, err_code) => {
                write!(f, "Failed calling {} with this error code: {}", fn_name, err_code)
            },
        }
    }
}

impl StdError for Error {
    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> { None }
}

/// A `FileDescriptor` that can be `StdIn`, `StdOut` or `StdErr`
/// Usefull when dealing with C call to [`ttyname`] and [`ttyname_r`]
///
/// [`ttyname`]: ../../../libc/fn.ttyname.html
/// [`ttyname_r`]: ../../../libc/fn.ttyname_r.html
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum FileDescriptor {
    StdIn  = 0,
    StdOut = 1,
    StdErr = 2,
}

impl FileDescriptor {
    /// Check if the `FileDescriptor` is a TTY.
    ///
    /// ## Example
    /// ```rust,ignore
    /// let istty = FileDescriptor::StdIn.is_tty();
    /// ```
    pub fn is_tty(self) -> bool { is_tty(self) }

    /// Get the [`TTYName`] from the `FileDescriptor`
    ///
    /// # Errors
    /// If it is not a TTY or a libc call fails and error variant is returned.
    ///
    /// [`TTYName`]: ./struct.TTYName.html
    pub fn ttyname(self) -> Result<TTYName, Error> { TTYName::new(self) }
}

/// A struct that holds the name of a TTY with a `Display` trait implementation
/// to be easy to print.
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct TTYName(BString);

impl TTYName {
    /// Create a `TTYName` from a `FileDescriptor`
    ///
    /// # Errors
    /// It returns a error variant when [`FileDescriptor`] is not a TTY.
    ///
    /// [`FileDescriptor`]: ./enum.FileDescriptor.html
    pub fn new(file_descriptor: FileDescriptor) -> Result<Self, Error> {
        let name = unsafe { ttyname(file_descriptor as c_int) };

        let name = if name.is_null() {
            return Err(Error::NotTTY);
        } else {
            let name_cstr = unsafe { CStr::from_ptr(name) };
            BString::from(name_cstr.to_bytes())
        };

        Ok(TTYName(name))
    }

    /// Extracts a bstring slice containing the entire `BString`.
    pub fn as_bstr(&self) -> &BStr { self.0.as_bstr() }

    /// Return a clone of the tty name.
    pub fn to_bstring(&self) -> BString { self.0.clone() }
}

impl Display for TTYName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Check if the given `FileDescriptor` is a TTY.
///
/// ## Example
/// ```rust,ignore
/// let istty = isatty(FileDescriptor::StdIn);
/// ```
#[inline]
pub fn is_tty(file_descriptor: FileDescriptor) -> bool {
    unsafe { libc::isatty(file_descriptor as c_int) == 1 }
}
