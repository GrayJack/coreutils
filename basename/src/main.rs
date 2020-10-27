mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let multiple_paths = matches.is_present("multiple") || matches.is_present("suffix");

    let suffix = matches.value_of("suffix").unwrap_or("");

    let line_ending = if matches.is_present("zero") { '\0' } else { '\n' };

    if multiple_paths {
        let paths = matches.values_of("NAME").unwrap();
        for path in paths {
            print!("{} {}", basename(path, suffix), line_ending);
        }
    } else {
        let path = matches.value_of("NAME").unwrap();
        print!("{} {}", basename(path, suffix), line_ending);
    }
}

/// Get `full_path` basename, removing the given `suffix`.
///
/// ## Examples:
/// ```rust
/// # fn main() {}
/// let name = basename("~/Pictures/mypicture.jpg", "");
/// assert_eq!("mypicture.jpg".to_string(), name);
/// # }
/// ```
///
/// ```rust
/// # fn main() {}
/// let name = basename("~/Pictures/mypicture.jpg", ".jpg");
/// assert_eq!("mypicture".to_string(), name);
/// # }
/// ```
fn basename(full_path: &str, suffix: &str) -> String {
    let split_full_path: Vec<&str> = full_path.split('/').collect();
    match split_full_path.last() {
        Some(name) => name.strip_suffix(suffix).unwrap_or(name).to_string(),
        None => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basename_empty_suffix_file() {
        assert_eq!("image.jpg".to_string(), basename("~/Pictures/image.jpg", ""));
        assert_eq!("doc.pdf".to_string(), basename("~/Documents/doc.pdf", ""));
    }

    #[test]
    fn basename_suffix_file() {
        assert_eq!("image".to_string(), basename("~/Pictures/image.jpg", ".jpg"));
        assert_eq!("doc".to_string(), basename("~/Documents/doc.pdf", ".pdf"));
    }

    #[test]
    fn basename_empty_suffix_dir() {
        assert_eq!("bin", basename("/usr/bin", ""));
        assert_eq!("Documents", basename("~/Documents", ""));
    }

    #[test]
    fn basename_suffix_dir() {
        assert_eq!("b", basename("/usr/bin", "in"));
        assert_eq!("Doc", basename("~/Documents", "uments"));
    }
}
