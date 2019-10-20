//! Account database module
use std::{
    collections::{hash_set, HashSet},
    fs::{self, File},
    io::{self, BufReader, Read},
    mem,
    path::Path,
    slice,
};

use crate::types::Time;

use libc::utmp;

use bstr::BString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Utmp {
    user: BString,
    line: BString,
    host: BString,
    time: Time,
}

impl Utmp {
    pub fn from_c_utmp(utm: utmp) -> Self {
        let user = {
            let cstr: String =
                utm.ut_name.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let host = {
            let cstr: String =
                utm.ut_host.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let line = {
            let cstr: String =
                utm.ut_line.iter().map(|cc| *cc as u8 as char).filter(|cc| cc != &'\0').collect();
            BString::from(cstr.as_bytes())
        };

        let time = utm.ut_time;

        Utmp { user, host, line, time }
    }
}

#[derive(Debug)]
pub struct UtmpSet(HashSet<Utmp>);

impl UtmpSet {
    /// Creates a new collection over a utmpx entry binary file
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
            set.insert(Utmp::from_c_utmp(raw_utm as utmp));
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
