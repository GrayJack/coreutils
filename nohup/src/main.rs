use std::{
    env,
    fs::{File, OpenOptions},
    io,
    os::{raw::c_int, unix::process::CommandExt},
    process::{self, Command, Stdio},
};

use coreutils_core::{
    libc::{signal, ENOENT, SIGHUP, SIG_IGN},
    tty::{FileDescriptor},
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("nohup.yml");
    let matches = App::from_yaml(yaml).settings(&[ColoredHelp]).get_matches();

    let command_name = matches.value_of("COMMAND").unwrap();
    let args = if matches.is_present("ARGS") {
        matches.values_of("ARGS").unwrap().collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut command = Command::new(command_name);
    let mut command_c = command.args(&args);

    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create(true).append(true);

    // If standard input is a terminal, redirect it from an unreadable file.
    if FileDescriptor::StdIn.is_tty() {
        command_c = command_c.stdin(Stdio::null());
    }

    if FileDescriptor::StdOut.is_tty() {
        // Try to open in write append nohup.out else open $HOME/nohup.out
        let stdout = match get_stdout(&open_opts) {
            Ok(f) => {
                println!("nohup: stdout is redirected to 'nohup.out'");
                f
            },
            Err(err) => {
                eprintln!("nohup: {}", err);
                process::exit(125);
            },
        };

        command_c = command_c.stdout(stdout);
    }

    // If standard error is a terminal, redirect it to standard output.
    if FileDescriptor::StdErr.is_tty() {
        let stderr = match get_stdout(&open_opts) {
            Ok(f) => {
                println!("nohup: stderr is redirected to 'nohup.out'");
                f
            },
            Err(err) => {
                eprintln!("nohup: {}", err);
                process::exit(125);
            },
        };

        command_c = command_c.stderr(stderr);
    }

    // Make all SIGHUP a ignored signal
    unsafe { signal(SIGHUP, SIG_IGN) };

    let err = command_c.exec();

    if err.raw_os_error().unwrap() as c_int == ENOENT {
        eprintln!("nohup: '{}': {}", command_name, err);
        process::exit(127);
    } else {
        eprintln!("nohup: {}", err);
        process::exit(126);
    }
}

fn get_stdout(open_opts: &OpenOptions) -> io::Result<File> {
    match open_opts.open("nohup.out") {
        Ok(file) => Ok(file),
        Err(_) => {
            let out = match env::var("HOME") {
                Ok(h) => {
                    let mut o = h;
                    o.push_str("nohup.out");
                    o
                },
                Err(err) => {
                    eprintln!("nohup: cannot replace STDOUT: {}", err);
                    process::exit(125)
                },
            };
            match open_opts.open(&out) {
                Ok(file) => {
                    println!("nohup: output is redirected to '{}'", out);
                    Ok(file)
                },
                Err(err) => {
                    eprintln!("nohup: here is no $HOME variable in the environment");
                    Err(err)
                },
            }
        },
    }
}
