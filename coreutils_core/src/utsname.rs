//! Module for system informayion
use std::{ffi::CStr, io, mem::MaybeUninit};

use bstr::{BStr, BString, ByteSlice};
use libc::{uname, utsname};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtsName {
    /// Name of the operating system implementation.
    sysname: BString,
    /// Network name of this machine.
    nodename: BString,
    /// Release level of the operating system.
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
    pub fn new() -> Result<Self, io::Error> {
        let mut uts_name: MaybeUninit<utsname> = MaybeUninit::zeroed();

        let res = unsafe { uname(uts_name.as_mut_ptr()) };

        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        let uts_name = unsafe { uts_name.assume_init() };

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

    pub fn system_name(&self) -> &BStr {
        self.sysname.as_bstr()
    }

    pub fn node_name(&self) -> &BStr {
        self.nodename.as_bstr()
    }

    pub fn release(&self) -> &BStr {
        self.release.as_bstr()
    }

    pub fn version(&self) -> &BStr {
        self.version.as_bstr()
    }

    pub fn machine(&self) -> &BStr {
        self.machine.as_bstr()
    }

    #[cfg(any(target_os = "linux", target_os = "fuchsia"))]
    pub fn domain_name(&self) -> &BStr {
        self.domainname.as_bstr()
    }
}
