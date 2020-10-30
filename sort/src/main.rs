use std::{
    fs::{self, File},
    io::{self, prelude::*},
};

use clap::ArgMatches;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    main_sort(matches).unwrap_or_else(|err| {
        eprintln!("sort: {:?}.", err);
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

fn get_inputs(matches: &clap::ArgMatches) -> Result<Vec<Box<dyn Read>>, SortError> {
    match matches.values_of("INPUT_FILES") {
        Some(files) => {
            let files: io::Result<Vec<File>> = files.map(File::open).collect();
            let files = files.map_err(SortError::FileReadError)?;

            let mut inputs: Vec<Box<dyn Read>> = vec![];
            for file in files {
                inputs.push(Box::new(file));
            }
            Ok(inputs)
        },
        None => Ok(vec![Box::new(io::stdin())]),
    }
}

fn sort(flags: &SortFlags, inputs: Vec<Box<dyn Read>>) -> Result<Vec<String>, SortError> {
    let lines: io::Result<Vec<String>> =
        inputs.into_iter().map(io::BufReader::new).flat_map(|reader| reader.lines()).collect();

    let mut lines = lines.map_err(SortError::FileReadError)?;
    if !flags.merge_only {
        lines.sort();
    }

    Ok(lines)
}

fn print_line(line: String, flags: &mut SortFlags) -> Result<(), SortError> {
    writeln!(flags.output, "{}", line).map_err(SortError::FileWriteError)
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
                Ok(file) => Box::new(file),
                Err(err) => return Err(SortError::FileWriteError(err)),
            },
            None => Box::new(io::stdout()),
        };
        Ok(SortFlags { merge_only, output })
    }
}

#[derive(Debug)]
enum SortError {
    FileReadError(io::Error),
    FileWriteError(io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
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
            Err(SortError::FileReadError(err)) if err.kind() == io::ErrorKind::NotFound => {},
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