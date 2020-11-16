use coreutils_core::BString;

/// Contains each row displayed in the -l` option.
pub(crate) type Table = Vec<Row>;

/// Contains each column displayed in the `-l` option.
pub(crate) struct Row {
    pub inode: String,
    pub block: String,
    pub permissions: String,
    pub hard_links: String,
    pub user: BString,
    pub group: BString,
    pub size: String,
    pub time: String,
    pub file_name: String,
}

impl Row {
    /// Initialize a new row
    pub fn new() -> Self {
        let inode = String::new();
        let block = String::new();
        let permissions = String::new();
        let hard_links = String::new();
        let user = BString::from("");
        let group = BString::from("");
        let size = String::new();
        let time = String::new();
        let file_name = String::new();

        Row { inode, block, permissions, hard_links, user, group, size, time, file_name }
    }
}
