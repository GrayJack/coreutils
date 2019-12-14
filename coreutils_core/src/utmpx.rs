//! Extended account database module
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
use std::{
    collections::{hash_set, HashSet},
    io,
    path::Path,
};
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    mem, slice,
};

use crate::types::{Pid, TimeVal};

#[cfg(any(target_os = "linux", target_os = "netbsd"))]
use libc::__exit_status as ExitStatus;
#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
use libc::c_int;
#[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
use libc::c_long;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use libc::utmpxname;
use libc::{endutxent, getutxent, setutxent, suseconds_t, time_t, utmpx};
#[cfg(target_os = "solaris")]
use libc::{exit_status as ExitStatus, short};

use bstr::{BStr, BString, ByteSlice};

use time::{Timespec, Tm};

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
    // Not sure yet
    DownTime,
    /// Invalid entry
    Invalid,
}

/// A struct that represents a __user__ account, where user can be humam users or other
/// parts of the system that requires the usage of account structure, like some daemons
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Utmpx {
    /// User login name
    user:    BString,
    /// Host name
    host:    BString,
    /// Process id creating the entry
    pid:     Pid,
    /// Record identifier (/etc/inittab id)
    id:      BString,
    /// Device name (console/tty, lnxx)
    line:    BString,
    /// Type of the entry
    ut_type: UtmpxType,
    /// The time entry was created
    timeval: TimeVal, // tv
    #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "solaris"))]
    exit:    ExitStatus,
    /// Session ID (used for windowing)
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
    session: c_int,
    /// Session ID (used for windowing)
    #[cfg(target_os = "solaris")]
    session: c_int,
    /// Session ID (used for windowing)
    #[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
    session: c_long,
    /// Session ID (used for windowing)
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    session: u16,
    #[cfg(target_os = "linux")]
    addr_v6: [i32; 4],
    #[cfg(target_os = "netbsd")]
    ss:      libc::sockaddr_storage,
    #[cfg(target_os = "solaris")]
    syslen:  c_short,
}

impl Utmpx {
    /// Creates a new `Utmpx` entry from the `C` version of the structure
    pub fn from_c_utmpx(utm: utmpx) -> Self { Self::from(utm) }

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
    pub const fn utype(&self) -> UtmpxType { self.ut_type }

    /// Get the time where the entry was created (often login time)
    pub const fn timeval(&self) -> TimeVal { self.timeval }

    /// Get the time where the entry was created (often login time) in a more complete
    /// structure
    pub fn login_time(&self) -> Tm {
        time::at(Timespec::new(self.timeval.tv_sec as i64, self.timeval.tv_usec as i32))
    }

    /// Get the session ID of the entry
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
    pub const fn session(&self) -> c_int { self.session }

    /// Get the session ID of the entry
    #[cfg(target_os = "solaris")]
    pub const fn session(&self) -> c_int { self.session }

    /// Get the session ID of the entry
    #[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
    pub const fn session(&self) -> c_long { self.session }

    /// Get the session ID of the entry
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    pub const fn session(&self) -> u16 { self.session }

    /// Get v6 address of the entry
    #[cfg(target_os = "linux")]
    pub const fn v6_addr(&self) -> [i32; 4] { self.addr_v6 }

    /// Get exit status of the entry
    #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "solaris"))]
    pub const fn exit_status(&self) -> ExitStatus { self.exit }
}

impl From<utmpx> for Utmpx {
    fn from(c_utmpx: utmpx) -> Self {
        #[cfg(not(any(target_os = "netbsd", target_os = "dragonfly")))]
        let user = {
            let cstr: String = c_utmpx
                .ut_user
                .iter()
                .map(|cc| *cc as u8 as char)
                .filter(|cc| cc != &'\0')
                .collect();
            BString::from(cstr.as_bytes())
        };

        #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
        let user = {
            let cstr: String = c_utmpx
                .ut_name
                .iter()
                .map(|cc| *cc as u8 as char)
                .filter(|cc| cc != &'\0')
                .collect();
            BString::from(cstr.as_bytes())
        };

        let host = {
            let cstr: String = c_utmpx
                .ut_host
                .iter()
                .map(|cc| *cc as u8 as char)
                .filter(|cc| cc != &'\0')
                .collect();
            BString::from(cstr.as_bytes())
        };

        let pid = c_utmpx.ut_pid;

        let id = {
            let cstr: String =
                c_utmpx.ut_id.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let line = {
            let cstr: String = c_utmpx
                .ut_line
                .iter()
                .map(|cc| *cc as u8 as char)
                .filter(|cc| cc != &'\0')
                .collect();
            BString::from(cstr.as_bytes())
        };

        let ut_type = UtmpxType::from(c_utmpx.ut_type);

        let timeval = TimeVal {
            tv_sec:  c_utmpx.ut_tv.tv_sec as time_t,
            tv_usec: c_utmpx.ut_tv.tv_usec as suseconds_t,
        };

        #[cfg(any(
            target_os = "linux",
            target_os = "netbsd",
            target_os = "dragonfly",
            target_os = "solaris"
        ))]
        let session = c_utmpx.ut_session;

        #[cfg(target_os = "linux")]
        let addr_v6 = c_utmpx.ut_addr_v6;

        #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "solaris"))]
        let exit = c_utmpx.ut_exit;

        #[cfg(any(target_os = "netbsd"))]
        let ss = c_utmpx.ut_ss;

        Utmpx {
            user,
            host,
            pid,
            id,
            line,
            ut_type,
            timeval,
            #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "solaris"))]
            exit,
            #[cfg(any(
                target_os = "linux",
                target_os = "netbsd",
                target_os = "dragonfly",
                target_os = "solaris"
            ))]
            session,
            #[cfg(target_os = "linux")]
            addr_v6,
            #[cfg(target_os = "netbsd")]
            ss,
        }
    }
}

/// A collection of Utmpx entries
#[derive(Debug)]
pub struct UtmpxSet(HashSet<Utmpx>);

impl UtmpxSet {
    /// Creates a new collection over a utmpx entry binary file
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = {
            let str = match path.as_ref().to_str() {
                Some(s) => s,
                None => "",
            };
            CString::new(str).unwrap_or_default()
        };

        let mut set = HashSet::new();

        unsafe {
            let res = utmpxname(file.as_ptr());

            if res != 0 {
                return Err(io::Error::last_os_error());
            }

            loop {
                let ut = getutxent();
                if ut.is_null() {
                    break;
                } else {
                    let utm = Utmpx::from_c_utmpx(*ut);
                    set.insert(utm);
                }
            }

            endutxent();
        }

        Ok(UtmpxSet(set))
    }

    /// Creates a new collection over a utmpx entry binary file
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let struct_size = mem::size_of::<utmpx>();
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
            set.insert(Utmpx::from_c_utmpx(raw_utm));
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
                if ut.is_null() {
                    break;
                } else {
                    let utm = Utmpx::from_c_utmpx(*ut);
                    set.insert(utm);
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

    /// Size of the collection
    pub fn len(&self) -> usize { self.0.len() }
}

impl IntoIterator for UtmpxSet {
    type IntoIter = hash_set::IntoIter<Utmpx>;
    type Item = Utmpx;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}


// Extra trait
macro_rules! utmpxtype_impl_from {
    ($($t:ty)+) => (
        $(
            #[cfg(target_os = "freebsd")]
            impl From<$t> for UtmpxType {
                fn from(num: $t) -> Self {
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

            #[cfg(target_os = "netbsd")]
            impl From<$t> for UtmpxType {
                fn from(num: $t) -> Self {
                    match num {
                        0 => Self::Empty,
                        1 => Self::RunLevel,
                        2 => Self::BootTime,
                        3 => Self::OldTime,
                        4 => Self::NewTime,
                        5 => Self::InitProcess,
                        6 => Self::LoginProcess,
                        7 => Self::UserProcess,
                        8 => Self::DeadProcess,
                        9 => Self::Accounting,
                        10 => Self::Signature,
                        11 => Self::DownTime,
                        _ => Self::Invalid,
                    }
                }
            }

            #[cfg(any(target_os = "dragonfly"))]
            impl From<$t> for UtmpxType {
                fn from(num: $t) -> Self {
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

            #[cfg(target_os = "solaris")]
            impl From<$t> for UtmpxType {
                fn from(num: $t) -> Self {
                    match num {
                        0 => Self::Empty,
                        1 => Self::RunLevel,
                        2 => Self::BootTime,
                        3 => Self::OldTime,
                        4 => Self::NewTime,
                        5 => Self::InitProcess,
                        6 => Self::LoginProcess,
                        7 => Self::UserProcess,
                        8 => Self::DeadProcess,
                        9 => Self::Accounting,
                        10 => Self::DownTime,
                        _ => Self::Invalid,
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            impl From<$t> for UtmpxType {
                fn from(num: $t) -> Self {
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


            #[cfg(target_os = "freebsd")]
            impl From<&$t> for UtmpxType {
                fn from(num: &$t) -> Self {
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

            #[cfg(target_os = "netbsd")]
            impl From<&$t> for UtmpxType {
                fn from(num: &$t) -> Self {
                    match num {
                        0 => Self::Empty,
                        1 => Self::RunLevel,
                        2 => Self::BootTime,
                        3 => Self::OldTime,
                        4 => Self::NewTime,
                        5 => Self::InitProcess,
                        6 => Self::LoginProcess,
                        7 => Self::UserProcess,
                        8 => Self::DeadProcess,
                        9 => Self::Accounting,
                        10 => Self::Signature,
                        11 => Self::DownTime,
                        _ => Self::Invalid,
                    }
                }
            }

            #[cfg(any(target_os = "dragonfly"))]
            impl From<&$t> for UtmpxType {
                fn from(num: &$t) -> Self {
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

            #[cfg(target_os = "solaris")]
            impl From<&$t> for UtmpxType {
                fn from(num: &$t) -> Self {
                    match num {
                        0 => Self::Empty,
                        1 => Self::RunLevel,
                        2 => Self::BootTime,
                        3 => Self::OldTime,
                        4 => Self::NewTime,
                        5 => Self::InitProcess,
                        6 => Self::LoginProcess,
                        7 => Self::UserProcess,
                        8 => Self::DeadProcess,
                        9 => Self::Accounting,
                        10 => Self::DownTime,
                        _ => Self::Invalid,
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            impl From<&$t> for UtmpxType {
                fn from(num: &$t) -> Self {
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

            #[cfg(target_os = "freebsd")]
            impl From<UtmpxType> for $t {
                fn from(utype: UtmpxType) -> Self {
                    match utype {
                        UtmpxType::Empty => 0,
                        UtmpxType::BootTime => 1,
                        UtmpxType::OldTime => 2,
                        UtmpxType::NewTime => 3,
                        UtmpxType::UserProcess => 4,
                        UtmpxType::InitProcess => 5,
                        UtmpxType::LoginProcess => 6,
                        UtmpxType::DeadProcess => 7,
                        UtmpxType::ShutdownProcess => 8,
                        UtmpxType::Invalid => 12,
                        _ => 12,
                    }
                }
            }

            #[cfg(target_os = "netbsd")]
            impl From<UtmpxType> for $t {
                fn from(utype: UtmpxType) -> Self {
                    match utype {
                        UtmpxType::Empty => 0,
                        UtmpxType::RunLevel => 1,
                        UtmpxType::BootTime => 2,
                        UtmpxType::OldTime => 3,
                        UtmpxType::NewTime => 4,
                        UtmpxType::InitProcess => 5,
                        UtmpxType::LoginProcess => 6,
                        UtmpxType::UserProcess => 7,
                        UtmpxType::DeadProcess => 8,
                        UtmpxType::Accounting => 9,
                        UtmpxType::Signature => 10,
                        UtmpxType::DownTime => 11,
                        UtmpxType::Invalid => 12,
                        _ => 12,
                    }
                }
            }

            #[cfg(any(target_os = "dragonfly"))]
            impl From<UtmpxType> for $t {
                fn from(utype: UtmpxType) -> Self {
                    match utype {
                        UtmpxType::Empty => 0,
                        UtmpxType::RunLevel => 1,
                        UtmpxType::BootTime => 2,
                        UtmpxType::NewTime => 3,
                        UtmpxType::OldTime => 4,
                        UtmpxType::InitProcess => 5,
                        UtmpxType::LoginProcess => 6,
                        UtmpxType::UserProcess => 7,
                        UtmpxType::DeadProcess => 8,
                        UtmpxType::Invalid => 12,
                        _ => 12,
                    }
                }
            }

            #[cfg(target_os = "solaris")]
            impl From<UtmpxType> for $t {
                fn from(utype: UtmpxType) -> Self {
                    match utype {
                        UtmpxType::Empty => 0,
                        UtmpxType::RunLevel => 1,
                        UtmpxType::BootTime => 2,
                        UtmpxType::OldTime => 3,
                        UtmpxType::NewTime => 4,
                        UtmpxType::InitProcess => 5,
                        UtmpxType::LoginProcess => 6,
                        UtmpxType::UserProcess => 7,
                        UtmpxType::DeadProcess => 8,
                        UtmpxType::Accounting => 9,
                        UtmpxType::DownTime => 10,
                        UtmpxType::Invalid => 12,
                        _ => 12,
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            impl From<UtmpxType> for $t {
                fn from(utype: UtmpxType) -> Self {
                    match utype {
                        UtmpxType::Empty => 0,
                        UtmpxType::RunLevel => 1,
                        UtmpxType::BootTime => 2,
                        UtmpxType::NewTime => 3,
                        UtmpxType::OldTime => 4,
                        UtmpxType::InitProcess => 5,
                        UtmpxType::LoginProcess => 6,
                        UtmpxType::UserProcess => 7,
                        UtmpxType::DeadProcess => 8,
                        #[cfg(any(target_os = "linux", target_os = "macos"))]
                        UtmpxType::Accounting => 9,
                        #[cfg(target_os = "macos")]
                        UtmpxType::Signature => 10,
                        #[cfg(target_os = "macos")]
                        UtmpxType::ShutdownProcess => 11,
                        UtmpxType::Invalid => 12,
                        _ => 12,
                    }
                }
            }
        )+
    );
}

utmpxtype_impl_from!(i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);
