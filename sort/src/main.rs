use std::{
    error, fmt,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    main_sort(matches).unwrap_or_else(|err| {
        eprintln!("sort: {}.", err);
        std::process::exit(2);
    });
}

fn main_sort(matches: clap::ArgMatches) -> Result<(), SortError> {
    let mut flags = SortFlags::from_matches(&matches)?;
    let inputs = get_inputs(&matches)?;

    let lines = sort(&flags, inputs)?;
    for line in lines {
        print_line(line, &mut flags)?;
    }

    Ok(())
}

fn get_inputs(matches: &clap::ArgMatches) -> Result<Vec<(String, Box<dyn Read>)>, SortError> {
    match matches.values_of("INPUT_FILES") {
        Some(files) => {
            let files = files.map(|path| {
                File::open(path)
                    .map(Box::new)
                    .map(|f| (path.to_string(), f))
                    .map_err(|err| SortError::read(path, err))
            });

            let mut inputs: Vec<(String, Box<dyn Read>)> = Vec::with_capacity(files.len());
            for file in files {
                let (s, f) = file?;
                inputs.push((s, f));
            }
            Ok(inputs)
        },
        None => Ok(vec![("stdin".to_string(), Box::new(io::stdin()))]),
    }
}

fn sort(flags: &SortFlags, inputs: Vec<(String, Box<dyn Read>)>) -> Result<Vec<String>, SortError> {
    let lines: Result<Vec<String>, _> = inputs
        .into_iter()
        .map(|(path, f)| (path, BufReader::new(f)))
        .flat_map(|(path, reader)| {
            reader.lines().map(move |res| res.map_err(|err| SortError::read(&path, err)))
        })
        .collect();

    let mut lines = lines?;
    if !flags.merge_only {
        lines.sort();
    }

    Ok(lines)
}

fn print_line(line: String, flags: &mut SortFlags) -> Result<(), SortError> {
    writeln!(flags.output, "{}", line).map_err(|err| SortError::write(&line, err))
}

struct SortFlags {
    merge_only: bool,
    output:     Box<dyn Write>,
}

impl SortFlags {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, SortError> {
        let merge_only = matches.is_present("merge_only");
        let output: Box<dyn Write> = match matches.value_of("OUTPUT_FILE") {
            Some(path) => match File::create(path) {
                Ok(file) => Box::new(BufWriter::new(file)),
                Err(err) => return Err(SortError::write(path, err)),
            },
            None => Box::new(BufWriter::new(io::stdout())),
        };
        Ok(SortFlags { merge_only, output })
    }
}

#[derive(Debug)]
struct SortError {
    path: String,
    ty:   SortErrorTy,
}

impl SortError {
    fn read(path: &str, err: io::Error) -> Self {
        SortError { path: path.to_string(), ty: SortErrorTy::FileReadError(err) }
    }

    fn write(path: &str, err: io::Error) -> Self {
        SortError { path: path.to_string(), ty: SortErrorTy::FileWriteError(err) }
    }
}

#[derive(Debug)]
enum SortErrorTy {
    FileReadError(io::Error),
    FileWriteError(io::Error),
}

impl fmt::Display for SortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            SortErrorTy::FileReadError(ref err) => {
                write!(f, "failed to read file {}: {}", self.path, err)
            },
            SortErrorTy::FileWriteError(ref err) => {
                write!(f, "failed to write file {}: {}", self.path, err)
            },
        }
    }
}

impl error::Error for SortError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.ty {
            SortErrorTy::FileReadError(ref err) => Some(err),
            SortErrorTy::FileWriteError(ref err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    macro_rules! create_temp_files {
        () => {{
            let mut file1 = NamedTempFile::new().unwrap();
            let mut file2 = NamedTempFile::new().unwrap();
            let mut file3 = NamedTempFile::new().unwrap();
            writeln!(file1, "file1").unwrap();
            writeln!(file2, "file2").unwrap();
            writeln!(file3, "file3").unwrap();

            (file1, file2, file3)
        }};
        ($content1:expr, $content2:expr, $content3:expr) => {{
            let mut file1 = NamedTempFile::new().unwrap();
            let mut file2 = NamedTempFile::new().unwrap();
            let mut file3 = NamedTempFile::new().unwrap();
            writeln!(file1, $content1).unwrap();
            writeln!(file2, $content2).unwrap();
            writeln!(file3, $content3).unwrap();

            (file1, file2, file3)
        }};
    }

    macro_rules! get_matches {
        ($file1:ident, $file2:ident, $file3:ident) => {
            cli::create_app().get_matches_from(vec![
                "sort",
                $file1.path().to_str().unwrap(),
                $file2.path().to_str().unwrap(),
                $file3.path().to_str().unwrap(),
            ]);
        };
        ($file1:ident, $file2:ident, $file3:ident, $output_path:ident) => {
            cli::create_app().get_matches_from(vec![
                "sort",
                "-o",
                $output_path.path().to_str().unwrap(),
                $file1.path().to_str().unwrap(),
                $file2.path().to_str().unwrap(),
                $file3.path().to_str().unwrap(),
            ]);
        };
        ($filePathAsString:ident) => {
            cli::create_app().get_matches_from(vec!["sort", $filePathAsString]);
        };
    }

    fn default_flags() -> SortFlags {
        SortFlags { merge_only: false, output: Box::new(io::stdout()) }
    }

    #[test]
    fn test_get_inputs_with_values() {
        let (file1, file2, file3) = create_temp_files!();
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();

        assert_eq!(3, inputs.len());

        let expected: Vec<String> =
            vec![file1, file2, file3].iter().map(|f| fs::read_to_string(f).unwrap()).collect();
        assert_eq!(vec!["file1\n", "file2\n", "file3\n"], expected);
    }

    #[test]
    fn test_get_inputs_fails() {
        let wrong_path = "/unexisting/path/I/hope";
        let matches = get_matches!(wrong_path);

        match get_inputs(&matches) {
            // Expected fail
            Err(SortError { ty: SortErrorTy::FileReadError(err), .. })
                if err.kind() == io::ErrorKind::NotFound => {},
            _ => panic!(),
        }
    }

    #[test]
    fn test_get_inputs_no_value() {
        let matches = cli::create_app().get_matches_from(vec!["sort"]);
        let inputs = get_inputs(&matches).unwrap();

        assert_eq!(1, inputs.len());
    }

    #[test]
    fn test_sort() {
        let (file1, file2, file3) = create_temp_files!("file3", "file2", "file1");
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();
        let res = sort(&default_flags(), inputs).unwrap();

        assert_eq!(vec!["file1", "file2", "file3"], res)
    }

    #[test]
    fn test_sort_multi_lines() {
        let (file1, file2, file3) =
            create_temp_files!("line1\nline3\nline4", "line8\nline2\nline5", "line6\nline7\nline9");
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();
        let res = sort(&default_flags(), inputs).unwrap();

        assert_eq!(
            vec!["line1", "line2", "line3", "line4", "line5", "line6", "line7", "line8", "line9"],
            res
        )
    }

    #[test]
    fn test_sort_different_file_length() {
        let (file1, file2, file3) =
            create_temp_files!("line1\nline3\nline4\nline8\nline2\nline5\nline6", "line7", "line9");
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();
        let res = sort(&default_flags(), inputs).unwrap();

        assert_eq!(
            vec!["line1", "line2", "line3", "line4", "line5", "line6", "line7", "line8", "line9"],
            res
        )
    }

    #[test]
    fn test_main_sort() {
        let output_file = NamedTempFile::new().unwrap();
        let output_file_path = output_file.path();

        let (file1, file2, file3) =
            create_temp_files!("line1\nline3\nline4\nline8\nline2\nline5\nline6", "line7", "line9");
        let matches = get_matches!(file1, file2, file3, output_file);

        main_sort(matches).unwrap();

        assert_eq!(
            vec![
                "line1", "line2", "line3", "line4", "line5", "line6", "line7", "line8", "line9", ""
            ]
            .join("\n"),
            fs::read_to_string(output_file_path).unwrap()
        )
    }
}
