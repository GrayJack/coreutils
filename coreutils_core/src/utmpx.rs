//! Extended account database module
use std::collections::{hash_set, HashSet};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::{io, ffi::CStr};

use crate::types::{Pid, TimeVal};

#[cfg(target_os = "linux")]
use libc::__exit_status;
#[cfg(not(any(target_os = "netbsd", target_os = "dragonfly")))]
use libc::c_short;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::utmpxname;
use libc::{endutxent, getutxent, setutxent, utmpx};

use bstr::{BStr, BString, ByteSlice};

/// Possible types of a `Utmpx` instance
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UtmpxType {
    /// Not sure yet (Linux and MacOS exclusive)
    Accounting,
    /// Time of a system boot.
    BootTime,
    /// A session leader exited.
    DeadProcess,
    /// No valid user accounting information.
    Empty,
    /// A process spawned by init(8).
    InitProcess,
    /// The session leader of a logged-in user.
    LoginProcess,
    /// Time after system clock change.
    NewTime,
    /// Time before system clock change.
    OldTime,
    /// Run level. Provided for compatibility, not used on NetBSD.
    RunLevel,
    /// Not sure yet (MacOS exclusive)
    Signature,
    /// The session leader of a time of system shutdown.
    ShutdownProcess,
    // A user process.
    UserProcess,
    /// Invalid entry
    Invalid,
}

#[cfg(target_os = "freebsd")]
impl From<c_short> for UtmpxType {
    fn from(num: c_short) -> Self {
        match num {
            0 => Self::Empty,
            1 => Self::BootTime,
            2 => Self::OldTime,
            3 => Self::NewTime,
            4 => Self::UserProcess,
            5 => Self::InitProcess,
            6 => Self::LoginProcess,
            7 => Self::DeadProcess,
            8 => Self::ShutdownProcess,
            _ => Self::Invalid,
        }
    }
}

#[cfg(not(any(target_os = "netbsd", target_os = "dragonfly", target_os = "freebsd")))]
impl From<c_short> for UtmpxType {
    fn from(num: c_short) -> Self {
        match num {
            0 => Self::Empty,
            1 => Self::RunLevel,
            2 => Self::BootTime,
            3 => Self::NewTime,
            4 => Self::OldTime,
            5 => Self::InitProcess,
            6 => Self::LoginProcess,
            7 => Self::UserProcess,
            8 => Self::DeadProcess,
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            9 => Self::Accounting,
            #[cfg(target_os = "macos")]
            10 => Self::Signature,
            #[cfg(target_os = "macos")]
            11 => Self::ShutdownProcess,
            _ => Self::Invalid,
        }
    }
}

#[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
impl From<u16> for UtmpxType {
    fn from(num: u16) -> Self {
        match num {
            0 => Self::Empty,
            1 => Self::RunLevel,
            2 => Self::BootTime,
            3 => Self::NewTime,
            4 => Self::OldTime,
            5 => Self::InitProcess,
            6 => Self::LoginProcess,
            7 => Self::UserProcess,
            8 => Self::DeadProcess,
            _ => Self::Invalid,
        }
    }
}

/// A struct that represents a __user__ account, where user can be humam users or other
/// parts of the system that requires the usage of account structure, like some daemons
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Utmpx {
    /// User login name
    user: BString,
    /// Host name
    host: BString,
    /// Process id creating the entry
    pid: Pid,
    /// Record identifier (/etc/inittab id)
    id: BString,
    /// Device name (console/tty, lnxx)
    line: BString,
    /// Type of the entry
    ut_type: UtmpxType,
    /// The time entry was created
    timeval: TimeVal, // tv
    #[cfg(target_os = "linux")]
    exit: __exit_status,
    /// Session ID (used for windowing)
    #[cfg(target_os = "linux")]
    session: i32,
    /// Session ID (used for windowing)
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    session: u16,
    #[cfg(target_os = "linux")]
    addr_v6: [i32; 4],
}

impl Utmpx {
    /// Creates a new `Utmpx` entry from the `C` version of the structure
    pub fn from_c_utmpx(utm: utmpx) -> Self {
        let user = {
            let cstr: String =
                utm.ut_user.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let host = {
            let cstr: String =
                utm.ut_host.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let pid = utm.ut_pid;

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

        let ut_type = UtmpxType::from(utm.ut_type);

        let timeval = utm.ut_tv;

        #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "dragonfly"))]
        let session = utm.ut_session;

        #[cfg(target_os = "linux")]
        let addr_v6 = utm.ut_addr_v6;

        #[cfg(target_os = "linux")]
        let exit = utm.ut_exit;

        Utmpx {
            user,
            host,
            pid,
            id,
            line,
            ut_type,
            timeval,
            #[cfg(target_os = "linux")]
            exit,
            #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "dragonfly"))]
            session,
            #[cfg(target_os = "linux")]
            addr_v6,
        }
    }

    /// Get user name
    pub fn user(&self) -> &BStr { self.user.as_bstr() }

    /// Get host name
    pub fn host(&self) -> &BStr { self.host.as_bstr() }

    /// Get the process ID
    pub fn process_id(&self) -> Pid { self.pid }

    /// Get the record ID
    pub fn id(&self) -> &BStr { self.id.as_bstr() }

    /// Get the device name of the entry (usually a tty or console)
    pub fn device_name(&self) -> &BStr { self.line.as_bstr() }

    /// Get the type kind if the entry
    pub fn utype(&self) -> UtmpxType { self.ut_type }

    /// Get the time where the entry was created
    pub fn timeval(&self) -> TimeVal { self.timeval }

    /// Get the session ID
    #[cfg(target_os = "linux")]
    pub fn session(&self) -> i32 { self.session }

    /// Get the session ID
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    pub fn session(&self) -> u16 { self.session }

    #[cfg(target_os = "linux")]
    pub fn v6_addr(&self) -> [i32; 4] { self.addr_v6 }
}

/// A collection of Utmpx entries
#[derive(Debug)]
pub struct UtmpxSet(HashSet<Utmpx>);

impl UtmpxSet {
    /// Creates a new collection over a utmpx entry binary file
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn from_file(file: &CStr) -> io::Result<Self> {
        let mut set = HashSet::new();

        unsafe {
            let res = utmpxname(file.as_ptr());

            if res != 0 {
                return Err(io::Error::last_os_error());
            }

            loop {
                let ut = getutxent();
                if !ut.is_null() {
                    let utm = Utmpx::from_c_utmpx(*ut);
                    set.insert(utm);
                } else {
                    break;
                }
            }

            endutxent();
        }

        Ok(UtmpxSet(set))
    }

    /// Creates a new collection geting all entries from the running system
    pub fn system() -> Self {
        let mut set = HashSet::new();

        unsafe {
            setutxent();

            loop {
                let ut = getutxent();
                if !ut.is_null() {
                    let utm = Utmpx::from_c_utmpx(*ut);
                    set.insert(utm);
                } else {
                    break;
                }
            }

            endutxent();
        }

        UtmpxSet(set)
    }

    /// Returns `true` if collection nas no elements
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    /// Creates a iterator over it's entries
    pub fn iter(&self) -> hash_set::Iter<'_, Utmpx> { self.0.iter() }
}

impl IntoIterator for UtmpxSet {
    type IntoIter = hash_set::IntoIter<Utmpx>;
    type Item = Utmpx;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
