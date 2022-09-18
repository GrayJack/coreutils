use std::{
    env::current_dir,
    fs::{self, FileType, Permissions},
    io,
    path::{Path, PathBuf},
    process,
};

use clap::ArgMatches;
use coreutils_core::input::*;

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    let flags = RmFlags::from_matches(&matches);

    let cwd = match current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("rm: error reading current working directory: {}", err);
            process::exit(1);
        },
    };

    // Safe to unwrap since we said it is required on clap configuration
    let files_relative: Vec<&str> = matches.values_of("FILE").unwrap().collect();

    // Safe to unwrap since we said it is required on clap configuration
    let files: Vec<PathBuf> = matches.values_of("FILE").unwrap().map(|s| cwd.join(s)).collect();

    if flags.preserve_root && files.contains(&PathBuf::from("/")) {
        eprintln!(
            "rm: it is dangerous to operate on '/', use --no-preserve-root to override this \
             failsafe."
        );
        process::exit(1);
    }

    if flags.interactive_batch && (files.len() > 3 || flags.recursive) {
        let is_affirmative = Input::new()
            .with_msg("rm: are you sure you want to do this deletion? [Y/n]: ")
            .is_affirmative();

        if !is_affirmative {
            process::exit(1);
        }
    }

    match rm(&files, &files_relative, flags) {
        Ok(()) => {},
        Err(msg) => {
            eprintln!("rm: {}", msg);
            process::exit(1);
        },
    };
}

#[derive(Debug, Clone, Copy)]
struct RmFlags {
    pub force: bool,
    pub interactive: bool,
    pub interactive_batch: bool,
    pub preserve_root: bool,
    pub recursive: bool,
    pub dirs: bool,
    pub verbose: bool,
}

impl RmFlags {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let force_index = matches.index_of("force").unwrap_or(0);
        let inter_index = matches.index_of("interactive").unwrap_or(0);
        let batch_index = matches.index_of("interactiveBatch").unwrap_or(0);

        let mut flags = RmFlags {
            force: matches.is_present("force"),
            interactive: matches.is_present("interactive"),
            interactive_batch: matches.is_present("interactiveBatch"),
            preserve_root: !matches.is_present("noPreserveRoot"),
            recursive: matches.is_present("recursive") | matches.is_present("recursive_compat"),
            dirs: matches.is_present("directories"),
            verbose: matches.is_present("verbose"),
        };

        if force_index > inter_index && force_index > batch_index {
            flags.interactive = false;
            flags.interactive_batch = false;
        }

        if inter_index > force_index && inter_index > batch_index {
            flags.force = false;
            flags.interactive_batch = false;
        }

        if batch_index > force_index && batch_index > inter_index {
            flags.force = false;
            flags.interactive = false;
        }

        flags
    }
}

fn ask(
    filetype: FileType, permissions: &Permissions, filename: &str, flags: RmFlags,
) -> Result<bool, ()> {
    if !flags.interactive && permissions.readonly() {
        if filetype.is_file() {
            let msg = format!("rm: delete write_protected regular file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        } else if filetype.is_dir() {
            let msg = format!("rm: delete write_protected dir file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        }
    }

    if flags.interactive {
        if filetype.is_file() && permissions.readonly() {
            let msg = format!("rm: delete write_protected regular file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        } else if filetype.is_file() && !permissions.readonly() {
            let msg = format!("rm: delete regular file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        } else if filetype.is_dir() && permissions.readonly() {
            let msg = format!("rm: delete write_protected directory file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        } else if filetype.is_dir() && !permissions.readonly() {
            let msg = format!("rm: delete directory file '{}'? [Y/n]: ", filename);
            return Ok(Input::new().with_msg(&msg).is_affirmative());
        }
    }

    Err(())
}

fn rm(files: &[PathBuf], relative: &[&str], flags: RmFlags) -> io::Result<()> {
    for (index, file) in files.iter().enumerate() {
        let metadata = file.metadata()?;
        let permissions = metadata.permissions();
        let filetype = metadata.file_type();

        if filetype.is_file() {
            if !flags.force && (flags.interactive ^ permissions.readonly()) {
                let is_affirmative = match ask(filetype, &permissions, relative[index], flags) {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!("rm: failed to get input when interactive of write protected");
                        process::exit(1);
                    },
                };

                if is_affirmative {
                    match fs::remove_file(file) {
                        Ok(()) => {
                            if flags.verbose {
                                println!("removed {}", file.display());
                            }
                        },
                        Err(err) => eprintln!(
                            "rm: cannot remove regular file '{}', {}",
                            relative[index], err
                        ),
                    };
                }
            } else {
                match fs::remove_file(file) {
                    Ok(()) => {
                        if flags.verbose {
                            println!("removed {}", file.display());
                        }
                    },
                    Err(err) => {
                        eprintln!("rm: cannot remove regular file '{}', {}", relative[index], err)
                    },
                }
            }
        } else if filetype.is_dir() {
            if flags.recursive {
                rm_dir_all(file, relative[index], filetype, &permissions, flags)?;
            } else if flags.dirs {
                if !flags.force && (flags.interactive ^ permissions.readonly()) {
                    let is_affirmative = match ask(filetype, &permissions, relative[index], flags) {
                        Ok(i) => i,
                        Err(_) => {
                            eprintln!(
                                "rm: failed to get input when interactive of write protected"
                            );
                            process::exit(1);
                        },
                    };

                    if is_affirmative {
                        match fs::remove_dir(file) {
                            Ok(()) => {
                                if flags.verbose {
                                    println!("removed {}", file.display());
                                }
                            },
                            Err(err) => eprintln!(
                                "rm: cannot remove directory file '{}': {}",
                                relative[index], err
                            ),
                        };
                    }
                } else {
                    match fs::remove_dir(file) {
                        Ok(()) => {
                            if flags.verbose {
                                println!("removed {}", file.display());
                            }
                        },
                        Err(err) => eprintln!(
                            "rm: cannot remove directory file '{}': {}",
                            relative[index], err
                        ),
                    };
                }
            } else {
                eprintln!("rm: cannot remove '{}': it is a directory", relative[index]);
            }
        } else {
            eprintln!("rm: cannot remove '{}': no such file or directory", relative[index]);
        }
    }
    Ok(())
}

fn rm_dir_all(
    file: &Path, relative: &str, filetype: FileType, permissions: &Permissions, flags: RmFlags,
) -> io::Result<()> {
    let file_type = fs::symlink_metadata(file)?.file_type();
    if file_type.is_symlink() {
        if !flags.force && (flags.interactive ^ permissions.readonly()) {
            let is_affirmative = match ask(filetype, permissions, relative, flags) {
                Ok(i) => i,
                Err(_) => {
                    eprintln!("rm: failed to get input when interactive of write protected");
                    process::exit(1);
                },
            };

            if is_affirmative {
                match fs::remove_file(file) {
                    Ok(()) => {
                        if flags.verbose {
                            println!("removed {}", file.display());
                        }
                    },
                    Err(err) => eprintln!("rm: cannot remove regular file '{}': {}", relative, err),
                }
            }
            Ok(())
        } else {
            match fs::remove_file(file) {
                Ok(()) => {
                    if flags.verbose {
                        println!("removed {}", file.display());
                    }
                },
                Err(err) => eprintln!("rm: cannot remove regular file '{}': {}", relative, err),
            };
            Ok(())
        }
    } else {
        rm_dir_all_recursive(file, relative, filetype, permissions, flags)
    }
}

fn rm_dir_all_recursive(
    file: &Path, relative: &str, filetype: FileType, permissions: &Permissions, flags: RmFlags,
) -> io::Result<()> {
    if flags.interactive {
        let msg = format!("rm: Descend into directory '{}'? [Y/n]: ", relative);
        let is_affirmative = Input::new().with_msg(&msg).is_affirmative();

        if !is_affirmative {
            return Ok(());
        }
    }

    for child in fs::read_dir(file)? {
        let child = child?;
        let child_permissions = child.metadata()?.permissions();
        let child_type = child.file_type()?;
        let child_relative = format!("{}/{}", relative, child.file_name().to_string_lossy());

        if child_type.is_dir() {
            rm_dir_all_recursive(
                &child.path(),
                &child_relative,
                child_type,
                &child_permissions,
                flags,
            )?
        } else if !flags.force && (flags.interactive || child_permissions.readonly()) {
            let is_affirmative = match ask(child_type, &child_permissions, &child_relative, flags) {
                Ok(i) => i,
                Err(_) => {
                    eprintln!("rm: failed to get input when interactive of write protected");
                    process::exit(1);
                },
            };

            if is_affirmative {
                match fs::remove_file(&child.path()) {
                    Ok(()) => {
                        if flags.verbose {
                            println!("removed {}", child.path().display());
                        }
                    },
                    Err(err) => {
                        eprintln!("rm: cannot remove regular file '{}': {}", child_relative, err)
                    },
                }
            }
        } else {
            match fs::remove_file(&child.path()) {
                Ok(()) => {
                    if flags.verbose {
                        println!("removed {}", child.path().display());
                    }
                },
                Err(err) => {
                    eprintln!("rm: cannot remove regular file '{}': {}", child_relative, err)
                },
            }
        }
    }

    if !flags.force && (flags.interactive ^ permissions.readonly()) {
        let is_affirmative = match ask(filetype, permissions, relative, flags) {
            Ok(i) => i,
            Err(_) => {
                eprintln!("rm: failed to get input when interactive of write protected");
                process::exit(1);
            },
        };

        if is_affirmative {
            match fs::remove_dir(file) {
                Ok(()) => {
                    if flags.verbose {
                        println!("removed {}", file.display());
                    }
                },
                Err(err) => eprintln!("rm: cannot remove directory file '{}': {}", relative, err),
            }
        }
    } else {
        match fs::remove_dir(file) {
            Ok(()) => {
                if flags.verbose {
                    println!("removed {}", file.display());
                }
            },
            Err(err) => eprintln!("rm: cannot remove directory file '{}': {}", relative, err),
        }
    }

    Ok(())
}
