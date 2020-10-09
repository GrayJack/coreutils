//! Account database module
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use std::convert::{TryFrom, TryInto};
use std::{
    collections::{hash_set, HashSet},
    fs::{self, File},
    io::{self, BufReader, Read},
    mem,
    path::Path,
    slice,
};

use super::Time;

use libc::{c_char, utmp};
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use libc::{c_short, exit_status as ExitStatus};
#[cfg(any(target_os = "netbsd", target_os = "solaris", target_os = "illumos"))]
use libc::{endutent, getutent, setutent};

use bstr::{BStr, BString, ByteSlice};
use time::OffsetDateTime as DataTime;

#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use super::utmpx::UtmpxKind;

/// A struct that represents a __user__ account, where user can be humam users or other
/// parts of the system that requires the usage of account structure, like some daemons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Utmp {
    /// User login name
    user:    BString,
    /// Device name (console/tty, lnxx)
    line:    BString,
    /// The time entry was created
    time:    Time,
    /// Host name
    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    host:    BString,
    /// Entry ID
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    id:      BString,
    /// Process ID
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    pid:     c_short,
    /// Entry type
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    ut_type: UtmpxKind,
    /// Exit status
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    exit:    ExitStatus,
}

impl Utmp {
    /// Creates a [`Utmp`] from the c structure [`utmp`].
    #[inline]
    pub fn from_c_utmp(utm: utmp) -> Self { Self::from(utm) }

    /// Get user name.
    #[inline]
    pub fn user(&self) -> &BStr { self.user.as_bstr() }

    /// Get host name.
    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    #[inline]
    pub fn host(&self) -> &BStr { self.host.as_bstr() }

    /// Get `/etc/inittab` id.
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[inline]
    pub fn id(&self) -> &BStr { self.id.as_bstr() }

    /// Get the device name of the entry. (usually a tty or console)
    #[inline]
    pub fn device_name(&self) -> &BStr { self.line.as_bstr() }

    /// Get the time the entry was created.
    #[inline]
    pub const fn time(&self) -> Time { self.time }

    /// Get the time where the entry was created (often login time) in a more complete
    /// structure.
    #[inline]
    pub fn login_time(&self) -> DataTime { DataTime::from_unix_timestamp(self.time) }

    /// Get the process ID of the entry.
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[inline]
    pub const fn pid(&self) -> c_short { self.pid }

    /// Get the entry type.
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[inline]
    pub const fn entry_type(&self) -> UtmpxKind { self.ut_type }

    /// Get the exit status of the entry.
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[inline]
    pub const fn exit_status(&self) -> ExitStatus { self.exit }
}

impl From<utmp> for Utmp {
    #[inline]
    fn from(utm: utmp) -> Self {
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        let user = {
            let cstr: String =
                utm.ut_name.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        let user = {
            let cstr: String =
                utm.ut_user.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        let host = {
            let cstr: String =
                utm.ut_host.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        let id = {
            let cstr: String =
                utm.ut_id.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let line = {
            let cstr: String =
                utm.ut_line.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let time = utm.ut_time;

        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        let ut_type = match UtmpxKind::try_from(utm.ut_type) {
            Ok(ut) => ut,
            Err(err) => panic!(format!("{}", err)),
        };

        Utmp {
            user,
            line,
            time,
            #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
            host,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            id,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            pid: utm.ut_pid,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            ut_type,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            exit: utm.ut_exit,
        }
    }
}

/// A collection of [`Utmp`].
#[derive(Debug)]
pub struct UtmpSet(HashSet<Utmp>);

impl UtmpSet {
    /// Creates a new collection over a [`utmp`] entry binary file.
    ///
    /// # Errors
    /// If a internal call set a errno (I/O OS error), an error variant will be returned.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let struct_size = mem::size_of::<utmp>();
        let num_bytes = fs::metadata(&path)?.len() as usize;
        let num_structs = num_bytes / struct_size;
        let mut reader = BufReader::new(File::open(&path)?);
        let mut vec = Vec::with_capacity(num_structs);
        let mut set = HashSet::with_capacity(num_structs);

        unsafe {
            let mut buffer = slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, num_bytes);
            reader.read_exact(&mut buffer)?;
            vec.set_len(num_structs);
        }

        for raw_utm in vec {
            set.insert(Utmp::from_c_utmp(raw_utm));
        }

        Ok(UtmpSet(set))
    }

    /// Creates a new collection geting all entries from the running system.
    ///
    /// # Errors
    /// If a internal call set a errno (I/O OS error), an error variant will be returned.
    #[inline]
    pub fn system() -> io::Result<Self> { Self::from_file("/var/run/utmp") }

    /// Returns `true` if collection nas no elements.
    #[inline]
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    /// Creates a iterator over it's entries.
    #[inline]
    pub fn iter(&self) -> hash_set::Iter<'_, Utmp> { self.0.iter() }
}

impl IntoIterator for UtmpSet {
    type IntoIter = hash_set::IntoIter<Utmp>;
    type Item = Utmp;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

/// Iterator over [`Utmp`]
#[cfg(any(target_os = "netbsd", target_os = "solaris", target_os = "illumos"))]
#[derive(Debug)]
pub struct UtmpIter;

#[cfg(any(target_os = "netbsd", target_os = "solaris", target_os = "illumos"))]
impl UtmpIter {
    /// Creates an iterator of the entries from the running system.
    #[inline]
    pub fn system() -> Self {
        unsafe { setutent() };
        Self
    }
}

#[cfg(any(target_os = "netbsd", target_os = "solaris", target_os = "illumos"))]
impl Iterator for UtmpIter {
    type Item = Utmp;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let ut = getutent();
            if ut.is_null() {
                endutent();
                None
            } else {
                let utm = Utmp::from(*ut);
                Some(utm)
            }
        }
    }
}

#[cfg(any(target_os = "netbsd", target_os = "solaris", target_os = "illumos"))]
impl std::iter::FusedIterator for UtmpIter {}

// Extra traits
#[cfg(target_os = "netbsd")]
pub(crate) mod consts {
    pub const USER_SIZE: usize = 8;
    pub const LINE_SIZE: usize = 8;
    pub const HOST_SIZE: usize = 16;
}

#[cfg(target_os = "solaris")]
pub(crate) mod consts {
    pub const USER_SIZE: usize = 8;
    pub const LINE_SIZE: usize = 12;
    pub const ID_SIZE: usize = 4;
}

#[cfg(target_os = "openbsd")]
pub(crate) mod consts {
    pub const USER_SIZE: usize = 32;
    pub const LINE_SIZE: usize = 8;
    pub const HOST_SIZE: usize = 256;
}


impl From<Utmp> for utmp {
    #[inline]
    fn from(utm: Utmp) -> Self {
        use self::consts::*;

        let mut ut_name = [0; USER_SIZE];
        let mut ut_line = [0; LINE_SIZE];
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        let mut ut_host = [0; HOST_SIZE];
        #[cfg(target_os = "solaris")]
        let mut ut_id = [0; ID_SIZE];

        utm.user.iter().enumerate().for_each(|(i, c)| ut_name[i] = *c as c_char);
        utm.line.iter().enumerate().for_each(|(i, c)| ut_line[i] = *c as c_char);
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        utm.host.iter().enumerate().for_each(|(i, c)| ut_host[i] = *c as c_char);
        #[cfg(target_os = "solaris")]
        utm.id.iter().enumerate().for_each(|(i, c)| ut_id[i] = *c as c_char);

        #[cfg(target_os = "solaris")]
        let ut_type = match utm.ut_type.try_into() {
            Ok(a) => a,
            Err(e) => panic!(format!("{}", e)),
        };

        utmp {
            #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
            ut_name,
            #[cfg(target_os = "solaris")]
            ut_user: ut_name,
            ut_line,
            ut_time: utm.time,
            #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
            ut_host,
            #[cfg(target_os = "solaris")]
            ut_id,
            #[cfg(target_os = "solaris")]
            ut_pid: utm.pid,
            #[cfg(target_os = "solaris")]
            ut_type,
            #[cfg(target_os = "solaris")]
            ut_exit: utm.exit,
        }
    }
}
