use std::{
    io::{self, Write},
    os::unix::fs::MetadataExt,
};

use coreutils_core::{
    os::tty::{is_tty, tty_dimensions},
    BString, ByteSlice,
};

use term_grid::{Alignment, Cell, Direction, Filling, Grid, GridOptions};

extern crate chrono;

use crate::{
    file::{FileColor, Files},
    flags::Flags,
    table::{Row, Table},
};

/// Writes the provided files in the default format.
pub(crate) fn default<W: Write>(files: Files, writer: &mut W, flags: Flags) -> io::Result<()> {
    if !is_tty(&io::stdout()) {
        for file in &files {
            writer.write_all(file.name.as_bytes())?;
            writeln!(writer)?;
        }

        return Ok(());
    } else if flags.one_per_line {
        for file in &files {
            let file_name = file.file_name(FileColor::Show);

            writeln!(writer, "{}", file_name)?;
        }

        return Ok(());
    } else if flags.comma_separate {
        for (i, file) in files.iter().enumerate() {
            let file_name = file.file_name(FileColor::Show);

            if (i + 1) == files.len() {
                writeln!(writer, "{}", file_name)?;
            } else {
                write!(writer, "{}, ", file_name)?;
            }
        }

        return Ok(());
    }

    Err(io::Error::new(io::ErrorKind::Other, "Failed to display files."))
}

/// Writes the provided files in a grid format.
pub(crate) fn grid<W: Write>(files: Files, writer: &mut W, direction: Direction) -> io::Result<()> {
    let mut grid = Grid::new(GridOptions { filling: Filling::Spaces(2), direction });

    let width = match tty_dimensions(&io::stdout()) {
        Some(result) => result.0,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to retrieve terminal dimensions.",
            ));
        },
    };

    for file in &files {
        grid.add(Cell {
            alignment: Alignment::Left,
            contents:  file.file_name(FileColor::Show),
            width:     file.file_name(FileColor::Hide).len(),
        });
    }

    match grid.fit_into_width(width.into()) {
        Some(display) => {
            write!(writer, "{}", display)?;
            Ok(())
        },
        None => {
            for file in &files {
                writeln!(writer, "{}", file.file_name(FileColor::Show))?;
            }

            Ok(())
        },
    }
}

/// Writes the provided files in a list format.
pub(crate) fn list<W: Write>(files: Files, writer: &mut W, flags: Flags) -> io::Result<()> {
    let mut inode_width = 1;
    let mut block_width = 1;
    let mut hard_links_width = 1;
    let mut user_width = 1;
    let mut group_width = 1;
    let mut size_width = 1;

    let mut rows = Table::new();

    let mut total: u64 = 0;

    for file in &files {
        let mut row = Row::new();

        // Process the file's inode
        if flags.inode {
            let inode = file.inode();
            let inode_len = inode.len();

            if inode_len > inode_width {
                inode_width = inode_len;
            }

            row.inode = inode;
        }

        total += file.blocks();

        // Process the file's block size
        if flags.size {
            let block = file.blocks() as usize;
            let block_len = block.to_string().len();

            if block_len > block_width {
                block_width = block_len;
            }

            row.block = block.to_string();
        }

        // Process the file's permissions
        let permissions = file.permissions();

        row.permissions = permissions;

        // Process the file's hard links
        let hard_links = file.hard_links();
        let hard_links_len = hard_links.len();

        if hard_links_len > hard_links_width {
            hard_links_width = hard_links_len;
        }

        row.hard_links = hard_links;

        // Process the file's user name
        if !flags.no_owner {
            let user = match file.user() {
                Ok(file_user) => file_user,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    BString::from(file.metadata.uid().to_string())
                },
            };

            let user_len = user.len();

            if user_len > user_width {
                user_width = user_len;
            }

            row.user = user;
        }

        // Process the file's group name
        if !flags.no_group {
            let group = match file.group() {
                Ok(file_group) => file_group,
                Err(err) => {
                    eprintln!("ls: {}", err);
                    BString::from(file.metadata.gid().to_string())
                },
            };

            let group_len = group.len();

            if group_len > group_width {
                group_width = group_len;
            }

            row.group = group;
        }

        // Process the file's size
        let size = file.size();
        let size_len = file.size().len();

        if size_len > size_width {
            size_width = size_len;
        }

        row.size = size;

        // Process the file's timestamp
        row.time = file.time();

        // Process the file's name
        row.file_name = file.file_name(FileColor::Show);

        rows.push(row);
    }

    if !flags.directory {
        writeln!(writer, "total {}", total)?;
    }

    for row in rows {
        if flags.inode {
            write!(writer, "{:>1$} ", row.inode, inode_width)?;
        }

        if flags.size {
            write!(writer, "{:>1$} ", row.block, block_width)?;
        }

        write!(writer, "{:<1} ", row.permissions)?;

        write!(writer, "{:>1$} ", row.hard_links, hard_links_width)?;

        if !flags.no_owner {
            write!(writer, "{:<1$} ", row.user.to_string(), user_width)?;
        }

        if !flags.no_group {
            write!(writer, "{:<1$} ", row.group.to_string(), group_width)?;
        }

        write!(writer, "{:>1$} ", row.size, size_width)?;

        write!(writer, "{:<1} ", row.time)?;

        write!(writer, "{:<1} ", row.file_name)?;

        writeln!(writer)?;
    }

    Ok(())
}
