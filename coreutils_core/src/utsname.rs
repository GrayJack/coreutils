//! Module for system information
use std::{
    convert::TryFrom,
    ffi::CStr,
    fmt::{self, Display},
    io,
    mem::MaybeUninit,
};

use bstr::{BStr, BString, ByteSlice};
use libc::{c_char, uname, utsname};

/// A struct that holds several system informations, like the system name, host name, etc.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtsName {
    /// Name of the operating system implementation.
    sysname: BString,
    /// Network name of this machine.
    nodename: BString,
    /// Release level of the operating system. (Often the kernel version)
    release: BString,
    /// Version level of the operating system.
    version: BString,
    /// Machine hardware platform.
    machine: BString,
    /// NIS or YP domain name
    #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
    domainname: BString,
}

impl UtsName {
    /// Generates a new `UtsName` of the system.
    pub fn new() -> Result<Self, io::Error> {
        let mut uts_name: MaybeUninit<utsname> = MaybeUninit::zeroed();

        let res = unsafe { uname(uts_name.as_mut_ptr()) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        let uts_name = unsafe { uts_name.assume_init() };

        Ok(UtsName::try_from(uts_name)?)
    }

    /// Get system name.
    #[inline]
    pub fn system_name(&self) -> &BStr { self.sysname.as_bstr() }

    /// Get host name of the machine
    #[inline]
    pub fn node_name(&self) -> &BStr { self.nodename.as_bstr() }

    /// Get the release level of the operating system.
    #[inline]
    pub fn release(&self) -> &BStr { self.release.as_bstr() }

    /// Get the version level of this release of the operating system.
    #[inline]
    pub fn version(&self) -> &BStr { self.version.as_bstr() }

    /// Get the type of the current hardware platform.
    #[inline]
    pub fn machine(&self) -> &BStr { self.machine.as_bstr() }

    /// NIS or YP domain name
    #[inline]
    #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
    pub fn domain_name(&self) -> &BStr { self.domainname.as_bstr() }
}

impl TryFrom<utsname> for UtsName {
    type Error = io::Error;

    fn try_from(uts_name: utsname) -> Result<Self, Self::Error> {
        let sysname = {
            let name = unsafe { CStr::from_ptr(&uts_name.sysname[0]) };
            BString::from(name.to_bytes())
        };

        let nodename = {
            let name = unsafe { CStr::from_ptr(&uts_name.nodename[0]) };
            BString::from(name.to_bytes())
        };

        let release = {
            let name = unsafe { CStr::from_ptr(&uts_name.release[0]) };
            BString::from(name.to_bytes())
        };

        let version = {
            let name = unsafe { CStr::from_ptr(&uts_name.version[0]) };
            BString::from(name.to_bytes())
        };

        let machine = {
            let name = unsafe { CStr::from_ptr(&uts_name.machine[0]) };
            BString::from(name.to_bytes())
        };

        #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
        let domainname = {
            let name = unsafe { CStr::from_ptr(&uts_name.domainname[0]) };
            BString::from(name.to_bytes())
        };

        Ok(UtsName {
            sysname,
            nodename,
            release,
            version,
            machine,
            #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
            domainname,
        })
    }
}

impl Display for UtsName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.sysname, self.nodename, self.release, self.version, self.machine
        )
    }
}

// Extra traits
#[cfg(any(
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "macos"
))]
const UTSNAME_SIZE: usize = 256;
#[cfg(any(target_os = "solaris"))]
const UTSNAME_SIZE: usize = 257;
#[cfg(any(target_os = "linux", target_os = "fuchsia"))]
const UTSNAME_SIZE: usize = 65;
#[cfg(any(target_os = "dragonfly", target_os = "haiku"))]
const UTSNAME_SIZE: usize = 32;

impl From<UtsName> for utsname {
    fn from(uts_name: UtsName) -> Self {
        let mut sysname = [0; UTSNAME_SIZE];
        let mut nodename = [0; UTSNAME_SIZE];
        let mut release = [0; UTSNAME_SIZE];
        let mut version = [0; UTSNAME_SIZE];
        let mut machine = [0; UTSNAME_SIZE];
        #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
        let mut domainname = [0; UTSNAME_SIZE];

        uts_name.sysname.iter().enumerate().for_each(|(i, c)| sysname[i] = *c as c_char);
        uts_name.nodename.iter().enumerate().for_each(|(i, c)| nodename[i] = *c as c_char);
        uts_name.release.iter().enumerate().for_each(|(i, c)| release[i] = *c as c_char);
        uts_name.version.iter().enumerate().for_each(|(i, c)| version[i] = *c as c_char);
        uts_name.machine.iter().enumerate().for_each(|(i, c)| machine[i] = *c as c_char);

        #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
        uts_name.domainname.iter().enumerate().for_each(|(i, c)| domainname[i] = *c as c_char);

        utsname {
            sysname,
            nodename,
            release,
            version,
            machine,
            #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
            domainname,
        }
    }
}
