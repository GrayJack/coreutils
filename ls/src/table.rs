/// Contains each row displayed in the -l` option.
pub(crate) struct Table {
    pub rows: Vec<Row>,
}

impl Table {
    /// Initialize a new table
    pub fn new() -> Self {
        let rows = Vec::new();

        Table { rows }
    }

    /// Add a row the list of table.
    pub fn push(&mut self, row: Row) { self.rows.push(row) }
}

/// Contains each column displayed in the `-l` option.
pub(crate) struct Row {
    pub inode: String,
    pub block: String,
    pub permissions: String,
    pub hard_links: String,
    pub user: String,
    pub group: String,
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
        let user = String::new();
        let group = String::new();
        let size = String::new();
        let time = String::new();
        let file_name = String::new();

        Row { inode, block, permissions, hard_links, user, group, size, time, file_name }
    }
}
