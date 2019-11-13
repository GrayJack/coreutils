use std::{env, fs, path::PathBuf, process};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

use coreutils_core::mktemp::{mkdtemp, mkstemp};

fn main() {
    let yaml = load_yaml!("mktemp.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let directory = matches.is_present("directory");
    let quiet = matches.is_present("quiet");
    let flag_t = matches.is_present("t");
    let unsafe_flag = matches.is_present("unsafe");

    // Construct a template appropriate for use with mkstemp/mkdtemp
    let template = if matches.is_present("TEMPLATE") {
        let template_arg = matches.value_of("TEMPLATE").unwrap();

        if flag_t {
            // Use value of environment value TMPDIR if set, otherwise /tmp
            let tmpdir = if let Ok(res) = env::var("TMPDIR") {
                PathBuf::from(res)
            } else {
                PathBuf::from("/tmp")
            };

            // If the template argument ends with X, treat it as a template appropriate for use
            // with mkstemp/mkdtemp.
            // If it does not, construct an appropriate template using the string we were passed as
            // a prefix.
            // This means the user will get a reasonable result, regardless of whether we're called
            // as if we were a BSD or GNU mktemp.
            if template_arg.ends_with('X') {
                tmpdir.join(template_arg)
            } else {
                tmpdir.join(format!("{}.XXXXXXXX", template_arg))
            }
        } else {
            // -t was not passed, use the passed template as-is
            PathBuf::from(template_arg)
        }
    } else {
        // No template was passed, use a sensible default
        PathBuf::from("/tmp/tmp.XXXXXXXX")
    };

    if directory {
        match mkdtemp(template.to_str().unwrap()) {
            Ok(dir) => {
                if unsafe_flag {
                    if let Err(rmerr) = fs::remove_dir(dir.clone()) {
                        if !quiet {
                            eprintln!(
                                "mktemp: Failed to remove temporary directory in unsafe mode: {}",
                                rmerr
                            );
                        }
                    }
                }
                println!("{}", dir);
            },
            Err(err) => {
                if !quiet {
                    eprintln!(
                        "mktemp: failed to create directory using template '{}': {}",
                        template.to_str().unwrap(),
                        err
                    );
                }
                process::exit(1);
            },
        }
    } else {
        // Create a file

        match mkstemp(template.to_str().unwrap()) {
            Ok(res) => {
                if unsafe_flag {
                    if let Err(rmerr) = fs::remove_file(res.path.clone()) {
                        if !quiet {
                            eprintln!(
                                "mktemp: Failed to remove temporary file in unsafe mode: {}",
                                rmerr
                            );
                        }
                    }
                }
                println!("{}", res);
            },
            Err(err) => {
                if !quiet {
                    eprintln!(
                        "mktemp: failed to create file using template '{}': {}",
                        template.to_str().unwrap(),
                        err
                    );
                }
                process::exit(1);
            },
        }
    }
}
