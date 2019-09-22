//! Module for environments abstractions.

use std::{
    convert::From,
    env::{self, VarError},
    error::Error as StdError,
    fmt::{self, Display},
    io::Error as IoError,
    mem::MaybeUninit,
    os::raw::c_char,
    path::PathBuf,
};

use libc::stat;

type Result<T> = std::result::Result<T, Error>;

/// Possible errors when calling this module functions
#[derive(Debug)]
pub enum Error {
    Var(VarError),
    Io(IoError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Var(err) => write!(f, "Failed to get var with error: {}", err),
            Self::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Error {
        Error::Var(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Var(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

/// Get the logical path of the current directory wrapped on a `Ok` if successful, returns a Err
/// holding the error that occurred.
pub fn current_dir_logical() -> Result<PathBuf> {
    let pwd = env::var("PWD")?;

    // Same as pwd, but null terminated
    let pwd_null = {
        let mut s = String::new();
        s.push_str(&pwd);
        s.push('\0');
        s
    };

    let (mut logical, mut physical) = (MaybeUninit::uninit(), MaybeUninit::uninit());

    // Validity check
    // if we can get both fisical and logical paths stat, check they are the same inode
    if pwd.starts_with('/') {
        let stat1 = unsafe { stat(pwd_null.as_ptr() as *const c_char, logical.as_mut_ptr()) == 0 };
        let stat2 = unsafe { stat(".\0".as_ptr() as *const c_char, physical.as_mut_ptr()) == 0 };

        let (logical, physical) = unsafe { (logical.assume_init(), physical.assume_init()) };

        if stat1 && stat2 && logical.st_ino == physical.st_ino {
            return Ok(PathBuf::from(pwd));
        }
    }
    Err(Error::Io(IoError::last_os_error()))
}
