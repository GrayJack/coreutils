use std::{
    error, fmt,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
};

use clap::ArgMatches;

mod cli;

type Buffer = Vec<u8>;

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

    let lines = sort(&flags, inputs);
    for line in lines {
        print_line(line, &mut flags)?;
    }

    Ok(())
}

fn get_inputs(matches: &clap::ArgMatches) -> Result<Vec<Buffer>, SortError> {
    match matches.values_of("INPUT_FILES") {
        Some(files) => {
            let files: Vec<_> = files
                .map(|path| {
                    File::open(path)
                        .map(BufReader::new)
                        .map(|file| {
                            file.split(b'\n')
                                .map(move |res| res.map_err(|err| SortError::read(path, err)))
                        })
                        .map_err(|err| SortError::read(path, err))
                })
                .collect::<Result<_, _>>()?;

            let cap = files.iter().fold(0_usize, |acc, v| acc + v.size_hint().0);
            // let mut inputs = Vec::with_capacity(cap);

            let inputs =
                files.into_iter().try_fold(Vec::with_capacity(cap), |mut inputs, mut lines| {
                    lines.try_for_each(|line| {
                        inputs.push(line?);
                        Ok(())
                    })?;
                    Ok(inputs)
                })?;

            Ok(inputs)
        },
        None => BufReader::new(io::stdin())
            .split(b'\n')
            .map(|res| res.map_err(|err| SortError::read("stdin", err)))
            .collect(),
    }
}

fn sort(flags: &SortFlags, mut inputs: Vec<Buffer>) -> Vec<Buffer> {
    if !flags.merge_only {
        inputs.sort_unstable();
    }

    inputs
}

fn print_line(line: Buffer, flags: &mut SortFlags) -> Result<(), SortError> {
    flags.output.write(&line).map_err(|err| SortError::write(&flags.output_name, err))?;
    writeln!(flags.output).map_err(|err| SortError::write(&flags.output_name, err))
}

struct SortFlags {
    merge_only:  bool,
    output_name: String,
    output:      Box<dyn Write>,
}

impl SortFlags {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, SortError> {
        let merge_only = matches.is_present("merge_only");
        let (output_name, output): (String, Box<dyn Write>) = match matches.value_of("OUTPUT_FILE")
        {
            Some(path) => match File::create(path) {
                Ok(file) => (path.to_string(), Box::new(BufWriter::new(file))),
                Err(err) => return Err(SortError::write(path, err)),
            },
            None => ("stdout".to_string(), Box::new(BufWriter::new(io::stdout()))),
        };
        Ok(SortFlags { merge_only, output_name, output })
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
        SortFlags {
            merge_only:  false,
            output_name: "stdout".to_string(),
            output:      Box::new(BufWriter::new(io::stdout())),
        }
    }

    #[test]
    fn test_get_inputs_with_values() {
        let (file1, file2, file3) = create_temp_files!();
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();

        assert_eq!(3, inputs.len());

        let expected: Vec<Buffer> =
            vec![file1, file2, file3].iter().map(|f| fs::read(f).unwrap()).collect();
        assert_eq!(vec![b"file1\n".to_vec(), b"file2\n".to_vec(), b"file3\n".to_vec()], expected);
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
    #[ignore]
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
        let res = sort(&default_flags(), inputs);

        assert_eq!(vec![b"file1".to_vec(), b"file2".to_vec(), b"file3".to_vec()], res)
    }

    #[test]
    fn test_sort_multi_lines() {
        let (file1, file2, file3) =
            create_temp_files!("line1\nline3\nline4", "line8\nline2\nline5", "line6\nline7\nline9");
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();
        let res = sort(&default_flags(), inputs);

        assert_eq!(
            vec![
                b"line1".to_vec(),
                b"line2".to_vec(),
                b"line3".to_vec(),
                b"line4".to_vec(),
                b"line5".to_vec(),
                b"line6".to_vec(),
                b"line7".to_vec(),
                b"line8".to_vec(),
                b"line9".to_vec()
            ],
            res
        )
    }

    #[test]
    fn test_sort_different_file_length() {
        let (file1, file2, file3) =
            create_temp_files!("line1\nline3\nline4\nline8\nline2\nline5\nline6", "line7", "line9");
        let matches = get_matches!(file1, file2, file3);

        let inputs = get_inputs(&matches).unwrap();
        let res = sort(&default_flags(), inputs);

        assert_eq!(
            vec![
                b"line1".to_vec(),
                b"line2".to_vec(),
                b"line3".to_vec(),
                b"line4".to_vec(),
                b"line5".to_vec(),
                b"line6".to_vec(),
                b"line7".to_vec(),
                b"line8".to_vec(),
                b"line9".to_vec()
            ],
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
            b"line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\n".to_vec(),
            fs::read(output_file_path).unwrap()
        )
    }
}
