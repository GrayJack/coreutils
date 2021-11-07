//! Extended account database module.
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "illumos"
))]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::net::{self, IpAddr};
use std::{
    collections::{hash_set, HashSet},
    error::Error as StdError,
    fmt::{self, Display},
    io,
    path::Path,
};
#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "illumos"
)))]
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    mem, slice,
};

use bstr::{BStr, BString, ByteSlice};
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
use libc::__exit_status as ExitStatus;
#[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
use libc::c_int;
#[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
use libc::c_long;
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "solaris",
    target_os = "illumos"
))]
use libc::utmpxname;
#[cfg(any(target_os = "solaris", target_os = "illumos"))]
use libc::{c_int, c_short, exit_status as ExitStatus};
use libc::{endutxent, getutxent, setutxent, suseconds_t, time_t, utmpx};
use time::{Duration, OffsetDateTime as DateTime};

use super::{Pid, TimeVal};

/// Error type for [`UtmpxKind`] conversion.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Error {
    /// When the OS has not this [`UtmpxKind`] and a number related to that kind.
    OsNoKind,
    /// When the OS has no [`UtmpxKind`] related to this number.
    OsNoNumber,
}

impl StdError for Error {}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OsNoKind => {
                write!(f, "This OS has not this UtmpxKind and a number related to that kind")
            },
            Self::OsNoNumber => write!(f, "This OS has no UtmpxKind related to this number"),
        }
    }
}

/// Possible types of a [`Utmpx`] instance.
#[repr(u16)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UtmpxKind {
    /// Not sure yet. (Linux and MacOS exclusive)
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
    /// Not sure yet. (MacOS exclusive)
    Signature,
    /// The session leader of a time of system shutdown.
    ShutdownProcess,
    // A user process.
    UserProcess,
    // Not sure yet.
    DownTime,
}

/// A struct that represents a __user__ account, where user can be humam users or other
/// parts of the system that requires the usage of account structure, like some daemons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Utmpx {
    /// User login name.
    user: BString,
    /// Host name.
    host: BString,
    /// Process id creating the entry.
    pid: Pid,
    /// Record identifier. (/etc/inittab id)
    id: BString,
    /// Device name. (console/tty, lnxx)
    line: BString,
    /// Type of the entry.
    ut_type: UtmpxKind,
    /// The time entry was created.
    timeval: TimeVal, // tv
    #[cfg(any(
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    ))]
    exit: ExitStatus,
    /// Session ID. (used for windowing)
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
    session: c_int,
    /// Session ID. (used for windowing)
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    session: c_int,
    /// Session ID. (used for windowing)
    #[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
    session: c_long,
    /// Session ID. (used for windowing)
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    session: u16,
    /// Internet address of remote host. Looks like that if it's IPV4 `addr_v6[0]` is
    /// non-zero and the rest is zero and if is IPV6 all indexes are non-zero.
    #[cfg(target_os = "linux")]
    addr_v6: [i32; 4],
    #[cfg(target_os = "netbsd")]
    ss: libc::sockaddr_storage,
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    syslen: c_short,
}

impl Utmpx {
    /// Get user name.
    #[inline]
    pub fn user(&self) -> &BStr {
        self.user.as_bstr()
    }

    /// Get host name.
    #[inline]
    pub fn host(&self) -> &BStr {
        self.host.as_bstr()
    }

    /// Get the process ID.
    #[inline]
    pub const fn process_id(&self) -> Pid {
        self.pid
    }

    /// Get the record ID.
    #[inline]
    pub fn id(&self) -> &BStr {
        self.id.as_bstr()
    }

    /// Get the device name of the entry (usually a tty or console).
    #[inline]
    pub fn device_name(&self) -> &BStr {
        self.line.as_bstr()
    }

    /// Get the type kind if the entry.
    #[inline]
    pub const fn entry_type(&self) -> UtmpxKind {
        self.ut_type
    }

    /// Get the time where the entry was created. (often login time)
    #[inline]
    pub const fn timeval(&self) -> TimeVal {
        self.timeval
    }

    /// Get the time where the entry was created (often login time) in a more complete
    /// structure.
    #[inline]
    pub fn login_time(&self) -> DateTime {
        DateTime::from_unix_timestamp(self.timeval.tv_sec as i64)
            + Duration::microseconds(self.timeval.tv_usec as i64)
    }

    /// Get the session ID of the entry.
    #[cfg(all(target_os = "linux", any(target_arch = "x86_64")))]
    #[inline]
    pub const fn session(&self) -> c_int {
        self.session
    }

    /// Get the session ID of the entry.
    #[cfg(any(target_os = "solaris", target_os = "illumos"))]
    #[inline]
    pub const fn session(&self) -> c_int {
        self.session
    }

    /// Get the session ID of the entry.
    #[cfg(all(target_os = "linux", not(any(target_arch = "x86_64"))))]
    #[inline]
    pub const fn session(&self) -> c_long {
        self.session
    }

    /// Get the session ID of the entry.
    #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
    #[inline]
    pub const fn session(&self) -> u16 {
        self.session
    }

    /// Get the IP address of the entry.
    #[cfg(target_os = "linux")]
    #[inline]
    pub fn address(&self) -> IpAddr {
        match self.addr_v6 {
            // In the man pages said that when it's IPV4, only the first number is set, otherwise it
            // is IPV6
            [x, 0, 0, 0] => IpAddr::V4(net::Ipv4Addr::from(x as u32)),
            [x, y, w, z] => {
                let x = x.to_be_bytes();
                let y = y.to_be_bytes();
                let w = w.to_be_bytes();
                let z = z.to_be_bytes();
                IpAddr::from([
                    x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3], w[0], w[1], w[2], w[3], z[0],
                    z[1], z[2], z[3],
                ])
            },
        }
    }

    /// Get exit status of the entry.
    #[cfg(any(
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    ))]
    #[inline]
    pub const fn exit_status(&self) -> ExitStatus {
        self.exit
    }
}

impl From<utmpx> for Utmpx {
    /// Converts [`utmpx`] to [`Utmpx`].
    ///
    /// # Panic
    /// This function may panic when converting a number to UtmpxKind. Since we get the
    /// number from the OS it should never panic, but if the OS drastically change, it
    /// may panic.
    #[inline]
    fn from(c_utmpx: utmpx) -> Self {
        #[cfg(not(any(target_os = "netbsd", target_os = "dragonfly")))]
        let user = {
            let cstr: Vec<_> =
                c_utmpx.ut_user.iter().map(|cc| *cc as u8).filter(|cc| cc != &b'\0').collect();
            BString::from(cstr)
        };

        #[cfg(any(target_os = "netbsd", target_os = "dragonfly"))]
        let user = {
            let cstr: Vec<_> =
                c_utmpx.ut_name.iter().map(|cc| *cc as u8).filter(|cc| cc != &b'\0').collect();
            BString::from(cstr)
        };

        let host = {
            let cstr: Vec<_> =
                c_utmpx.ut_host.iter().map(|cc| *cc as u8).filter(|cc| cc != &b'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let pid = c_utmpx.ut_pid;

        let id = {
            let cstr: Vec<_> =
                c_utmpx.ut_id.iter().map(|cc| *cc as u8).filter(|cc| cc != &b'\0').collect();
            BString::from(cstr)
        };

        let line = {
            let cstr: Vec<_> =
                c_utmpx.ut_line.iter().map(|cc| *cc as u8).filter(|cc| cc != &b'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let ut_type = match UtmpxKind::try_from(c_utmpx.ut_type) {
            Ok(ut) => ut,
            Err(err) => panic!("{}", err),
        };

        let timeval = TimeVal {
            tv_sec: c_utmpx.ut_tv.tv_sec as time_t,
            tv_usec: c_utmpx.ut_tv.tv_usec as suseconds_t,
        };

        #[cfg(any(
            target_os = "linux",
            target_os = "netbsd",
            target_os = "dragonfly",
            target_os = "solaris",
            target_os = "illumos"
        ))]
        let session = c_utmpx.ut_session;

        #[cfg(target_os = "linux")]
        let addr_v6 = c_utmpx.ut_addr_v6;

        #[cfg(any(
            target_os = "linux",
            target_os = "netbsd",
            target_os = "solaris",
            target_os = "illumos"
        ))]
        let exit = c_utmpx.ut_exit;

        #[cfg(any(target_os = "netbsd"))]
        let ss = c_utmpx.ut_ss;

        #[cfg(any(target_os = "solaris", target_os = "illumos"))]
        let syslen = c_utmpx.ut_syslen;

        Utmpx {
            user,
            host,
            pid,
            id,
            line,
            ut_type,
            timeval,
            #[cfg(any(
                target_os = "linux",
                target_os = "netbsd",
                target_os = "solaris",
                target_os = "illumos"
            ))]
            exit,
            #[cfg(any(
                target_os = "linux",
                target_os = "netbsd",
                target_os = "dragonfly",
                target_os = "solaris",
                target_os = "illumos"
            ))]
            session,
            #[cfg(target_os = "linux")]
            addr_v6,
            #[cfg(target_os = "netbsd")]
            ss,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            syslen,
        }
    }
}

/// A collection of Utmpx entries.
#[derive(Debug)]
pub struct UtmpxSet(HashSet<Utmpx>);

impl UtmpxSet {
    /// Creates a new collection over a utmpx entry binary file.
    ///
    /// # Errors
    /// If a internal call set a errno (I/O OS error), an error variant will be returned.
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    ))]
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = {
            let str = path.as_ref().to_str().unwrap_or("");
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
                    let utm = Utmpx::from(*ut);
                    set.insert(utm);
                }
            }

            endutxent();
        }

        Ok(UtmpxSet(set))
    }

    /// Creates a new collection over a utmpx entry binary file.
    ///
    /// # Errors
    /// If a internal call set a errno (I/O OS error), an error variant will be returned.
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    )))]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let struct_size = mem::size_of::<utmpx>();
        let num_bytes = fs::metadata(&path)?.len() as usize;
        let num_structs = num_bytes / struct_size;
        let mut reader = BufReader::new(File::open(&path)?);
        let mut vec: Vec<utmpx> = Vec::with_capacity(num_structs);
        let mut set = HashSet::with_capacity(num_structs);

        unsafe {
            let mut buffer = slice::from_raw_parts_mut(vec.as_mut_ptr() as *mut u8, num_bytes);
            reader.read_exact(&mut buffer)?;
            vec.set_len(num_structs);
        }

        for raw_utm in vec {
            set.insert(Utmpx::from(raw_utm));
        }

        Ok(UtmpxSet(set))
    }

    /// Creates a new collection geting all entries from the running system.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn system() -> Self {
        let mut set = HashSet::new();

        unsafe {
            setutxent();

            loop {
                let ut = getutxent();
                if ut.is_null() {
                    break;
                } else {
                    let utm = Utmpx::from(*ut);
                    set.insert(utm);
                }
            }

            endutxent();
        }

        UtmpxSet(set)
    }

    /// Returns `true` if collection nas no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Creates a iterator over it's entries.
    #[inline]
    pub fn iter(&self) -> hash_set::Iter<'_, Utmpx> {
        self.0.iter()
    }

    /// Size of the collection.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for UtmpxSet {
    type IntoIter = hash_set::IntoIter<Utmpx>;
    type Item = Utmpx;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A iterator over the [`Utmpx`].
#[derive(Debug)]
pub struct UtmpxIter;

impl UtmpxIter {
    /// Creates an iterator of the entries from the running system.
    #[inline]
    pub fn system() -> Self {
        unsafe { setutxent() };
        Self
    }

    /// Creates an iterator over a utmpx entry binary file.
    ///
    /// # Errors
    /// If a internal call set a errno (I/O OS error), an error variant will be returned.
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "solaris",
        target_os = "illumos"
    ))]
    #[inline]
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = {
            let str = path.as_ref().to_str().unwrap_or("");
            CString::new(str).unwrap_or_default()
        };

        let res = unsafe { utmpxname(file.as_ptr()) };
        if res != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self)
    }
}


impl Iterator for UtmpxIter {
    type Item = Utmpx;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let ut = getutxent();
            if ut.is_null() {
                endutxent();
                None
            } else {
                let utm = Utmpx::from(*ut);
                Some(utm)
            }
        }
    }
}

impl std::iter::FusedIterator for UtmpxIter {}

// Extra trait
macro_rules! utmpxkind_impl_from {
    ($($t:ty)+) => (
        $(
            // Number to UtmpxKind
            #[cfg(target_os = "freebsd")]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::BootTime),
                        2 => Ok(Self::OldTime),
                        3 => Ok(Self::NewTime),
                        4 => Ok(Self::UserProcess),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::DeadProcess),
                        8 => Ok(Self::ShutdownProcess),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            #[cfg(target_os = "netbsd")]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::RunLevel),
                        2 => Ok(Self::BootTime),
                        3 => Ok(Self::OldTime),
                        4 => Ok(Self::NewTime),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::UserProcess),
                        8 => Ok(Self::DeadProcess),
                        9 => Ok(Self::Accounting),
                        10 => Ok(Self::Signature),
                        11 => Ok(Self::DownTime),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            #[cfg(any(target_os = "dragonfly"))]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::RunLevel),
                        2 => Ok(Self::BootTime),
                        3 => Ok(Self::NewTime),
                        4 => Ok(Self::OldTime),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::UserProcess),
                        8 => Ok(Self::DeadProcess),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::RunLevel),
                        2 => Ok(Self::BootTime),
                        3 => Ok(Self::OldTime),
                        4 => Ok(Self::NewTime),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::UserProcess),
                        8 => Ok(Self::DeadProcess),
                        9 => Ok(Self::Accounting),
                        10 => Ok(Self::DownTime),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::RunLevel),
                        2 => Ok(Self::BootTime),
                        3 => Ok(Self::NewTime),
                        4 => Ok(Self::OldTime),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::UserProcess),
                        8 => Ok(Self::DeadProcess),
                        9 => Ok(Self::Accounting),
                        #[cfg(target_os = "macos")]
                        10 => Ok(Self::Signature),
                        #[cfg(target_os = "macos")]
                        11 => Ok(Self::ShutdownProcess),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            #[cfg(target_os = "haiku")]
            impl TryFrom<$t> for UtmpxKind {
                type Error = Error;
                #[inline]
                fn try_from(num: $t) -> Result<Self, Error> {
                    match num {
                        0 => Ok(Self::Empty),
                        1 => Ok(Self::BootTime),
                        2 => Ok(Self::OldTime),
                        3 => Ok(Self::NewTime),
                        4 => Ok(Self::UserProcess),
                        5 => Ok(Self::InitProcess),
                        6 => Ok(Self::LoginProcess),
                        7 => Ok(Self::DeadProcess),
                        _ => Err(Error::OsNoNumber),
                    }
                }
            }

            // UtmpxKind to number
            #[cfg(target_os = "freebsd")]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::BootTime => Ok(1),
                        UtmpxKind::OldTime => Ok(2),
                        UtmpxKind::NewTime => Ok(3),
                        UtmpxKind::UserProcess => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::DeadProcess => Ok(7),
                        UtmpxKind::ShutdownProcess => Ok(8),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }

            #[cfg(target_os = "netbsd")]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::RunLevel => Ok(1),
                        UtmpxKind::BootTime => Ok(2),
                        UtmpxKind::OldTime => Ok(3),
                        UtmpxKind::NewTime => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::UserProcess => Ok(7),
                        UtmpxKind::DeadProcess => Ok(8),
                        UtmpxKind::Accounting => Ok(9),
                        UtmpxKind::Signature => Ok(10),
                        UtmpxKind::DownTime => Ok(11),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }

            #[cfg(any(target_os = "dragonfly"))]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::RunLevel => Ok(1),
                        UtmpxKind::BootTime => Ok(2),
                        UtmpxKind::NewTime => Ok(3),
                        UtmpxKind::OldTime => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::UserProcess => Ok(7),
                        UtmpxKind::DeadProcess => Ok(8),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }

            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::RunLevel => Ok(1),
                        UtmpxKind::BootTime => Ok(2),
                        UtmpxKind::OldTime => Ok(3),
                        UtmpxKind::NewTime => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::UserProcess => Ok(7),
                        UtmpxKind::DeadProcess => Ok(8),
                        UtmpxKind::Accounting => Ok(9),
                        UtmpxKind::DownTime => Ok(10),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "macos"))]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::RunLevel => Ok(1),
                        UtmpxKind::BootTime => Ok(2),
                        UtmpxKind::NewTime => Ok(3),
                        UtmpxKind::OldTime => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::UserProcess => Ok(7),
                        UtmpxKind::DeadProcess => Ok(8),
                        UtmpxKind::Accounting => Ok(9),
                        #[cfg(target_os = "macos")]
                        UtmpxKind::Signature => Ok(10),
                        #[cfg(target_os = "macos")]
                        UtmpxKind::ShutdownProcess => Ok(11),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }

            #[cfg(target_os = "haiku")]
            impl TryFrom<UtmpxKind> for $t {
                type Error = Error;
                #[inline]
                fn try_from(utype: UtmpxKind) -> Result<Self, Error> {
                    match utype {
                        UtmpxKind::Empty => Ok(0),
                        UtmpxKind::BootTime => Ok(1),
                        UtmpxKind::OldTime => Ok(2),
                        UtmpxKind::NewTime => Ok(3),
                        UtmpxKind::UserProcess => Ok(4),
                        UtmpxKind::InitProcess => Ok(5),
                        UtmpxKind::LoginProcess => Ok(6),
                        UtmpxKind::DeadProcess => Ok(7),
                        _ => Err(Error::OsNoKind),
                    }
                }
            }
        )+
    );
}

utmpxkind_impl_from!(i8 i16 i32 i64 i128 u8 u16 u32 u64 u128);
