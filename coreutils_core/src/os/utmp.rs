//! Account database module
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

use bstr::{BStr, BString, ByteSlice};
use time::PrimitiveDateTime as DataTime;

#[cfg(target_os = "solaris")]
use crate::utmpx::UtmpxType;
#[cfg(target_os = "solaris")]
use libc::{c_short, exit_status as ExitStatus};

/// A struct that represents a __user__ account, where user can be humam users or other
/// parts of the system that requires the usage of account structure, like some daemons
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
    #[cfg(target_os = "solaris")]
    id:      BString,
    /// Process ID
    #[cfg(target_os = "solaris")]
    pid:     c_short,
    /// Entry type
    #[cfg(target_os = "solaris")]
    ut_type: UtmpxType,
    /// Exit status
    #[cfg(target_os = "solaris")]
    exit:    ExitStatus,
}

impl Utmp {
    /// Creates a `Utmp` from the c structure `utmp`
    pub fn from_c_utmp(utm: utmp) -> Self { Self::from(utm) }

    /// Get user name
    pub fn user(&self) -> &BStr { self.user.as_bstr() }

    /// Get host name
    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    pub fn host(&self) -> &BStr { self.host.as_bstr() }

    /// Get `/etc/inittab` id
    #[cfg(target_os = "solaris")]
    pub fn id(&self) -> &BStr { self.id.as_bstr() }

    /// Get the device name of the entry (usually a tty or console)
    pub fn device_name(&self) -> &BStr { self.line.as_bstr() }

    /// Get the time the entry was created
    pub const fn time(&self) -> Time { self.time }

    /// Get the time where the entry was created (often login time) in a more complete
    /// structure
    pub fn login_time(&self) -> DataTime { DataTime::from_unix_timestamp(self.time) }

    /// Get the process ID of the entry
    #[cfg(target_os = "solaris")]
    pub fn pid(&self) -> c_short { self.pid }

    /// Get the entry type
    #[cfg(target_os = "solaris")]
    pub fn entry_type(&self) -> UtmpxType { self.ut_type }

    /// Get the exit status of the entry
    #[cfg(target_os = "solaris")]
    pub fn exit_status(&self) -> ExitStatus { self.exit }
}

impl From<utmp> for Utmp {
    fn from(utm: utmp) -> Self {
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        let user = {
            let cstr: String =
                utm.ut_name.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        #[cfg(target_os = "solaris")]
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

        #[cfg(target_os = "solaris")]
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

        Utmp {
            user,
            line,
            time,
            #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
            host,
            #[cfg(target_os = "solaris")]
            id,
            #[cfg(target_os = "solaris")]
            pid: utm.ut_pid,
            #[cfg(target_os = "solaris")]
            ut_type: UtmpxType::from(utm.ut_type),
            #[cfg(target_os = "solaris")]
            exit: utm.ut_exit,
        }
    }
}

#[derive(Debug)]
pub struct UtmpSet(HashSet<Utmp>);

impl UtmpSet {
    /// Creates a new collection over a `utmpx` entry binary file
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

    /// Creates a new collection geting all entries from the running system
    pub fn system() -> io::Result<Self> { Self::from_file("/var/run/utmp") }

    /// Returns `true` if collection nas no elements
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    /// Creates a iterator over it's entries
    pub fn iter(&self) -> hash_set::Iter<'_, Utmp> { self.0.iter() }
}

impl IntoIterator for UtmpSet {
    type IntoIter = hash_set::IntoIter<Utmp>;
    type Item = Utmp;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

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
            ut_type: utm.ut_type.into(),
            #[cfg(target_os = "solaris")]
            ut_exit: utm.exit,
        }
    }
}
