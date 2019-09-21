#![allow(non_camel_case_types)]
use std::{
    fmt::{self, Display},
    mem::MaybeUninit,
};

use libc::{c_int, c_uint, dev_t, pid_t, uid_t};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct AuditError {
    err: String,
}

impl Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub type au_id_t = uid_t;
pub type au_asid_t = pid_t;
pub type au_event_t = c_uint;
pub type au_emod_t = c_uint;
pub type au_class_t = c_int;

#[derive(Debug)]
#[repr(C)]
pub struct au_mask {
    pub am_success: c_uint,
    pub am_failure: c_uint,
}
pub type au_mask_t = au_mask;

#[derive(Debug)]
#[repr(C)]
pub struct au_tid_addr {
    pub port: dev_t,
}
pub type au_tid_addr_t = au_tid_addr;

#[derive(Debug)]
#[repr(C)]
pub struct c_auditinfo_addr {
    pub ai_auid: au_id_t,         // Audit user ID
    pub ai_mask: au_mask_t,       // Audit masks.
    pub ai_termid: au_tid_addr_t, // Terminal ID.
    pub ai_asid: au_asid_t,       // Audit session ID.
    pub ai_flags: u64,            // Audit session flags
}
pub type c_auditinfo_addr_t = c_auditinfo_addr;

extern "C" {
    pub fn getaudit(auditinfo_addr: *mut c_auditinfo_addr_t) -> c_int;
}

pub fn auditid() -> Result<(), AuditError> {
    let mut auditinfo: MaybeUninit<c_auditinfo_addr_t> = MaybeUninit::zeroed();
    let address = auditinfo.as_mut_ptr() as *mut c_auditinfo_addr_t;

    if unsafe { getaudit(address) } < 0 {
        return Err(AuditError {
            err: "Couldn't retrieve information".to_string(),
        });
    }

    let auditinfo = unsafe { auditinfo.assume_init() };

    println!("auid={}", auditinfo.ai_auid);
    println!("mask.success=0x{:x}", auditinfo.ai_mask.am_success);
    println!("mask.failure=0x{:x}", auditinfo.ai_mask.am_failure);
    println!("termid.port=0x{:x}", auditinfo.ai_termid.port);
    println!("asid={}", auditinfo.ai_asid);
    Ok(())
}
