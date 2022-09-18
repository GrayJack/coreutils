use std::path::Path;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let line_ending = if matches.is_present("zero") { '\0' } else { '\n' };

    // We know that NAME is required, so it's ok to unwrap
    let paths = matches.values_of("NAME").unwrap();
    for path in paths {
        print!("{} {}", dirname(path), line_ending)
    }
}

/// Get the directory full name of a given `path`.
///
/// ```rust
/// let path = "/home/user/";
/// assert_eq!("/home".to_string(), dirname(path));
/// ```
fn dirname(path: &str) -> String {
    let p = Path::new(path);
    match p.parent() {
        Some(dir) => {
            if dir.components().next().is_none() {
                ".".to_string()
            } else {
                dir.to_string_lossy().to_string()
            }
        },
        None => {
            if p.is_absolute() || path == "/" {
                "/".to_string()
            } else {
                ".".to_string()
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirname_root_path() {
        assert_eq!("/".to_string(), dirname("/"));
        assert_eq!("/".to_string(), dirname("/usr"));
    }

    #[test]
    fn dirname_absolute_path() {
        assert_eq!(".".to_string(), dirname("doc"));
        assert_eq!(".".to_string(), dirname("doc"));
    }

    #[test]
    fn dirname_not_absolute_dir() {
        assert_eq!("/home".to_string(), dirname("/home/user/"));
        assert_eq!("somedir".to_string(), dirname("somedir/anotherdir/"));
    }

    #[test]
    fn dirname_not_absolute_file() {
        assert_eq!("/usr/bin".to_string(), dirname("/usr/bin/zsh"));
        assert_eq!("dir".to_string(), dirname("dir/file"));
    }
}
