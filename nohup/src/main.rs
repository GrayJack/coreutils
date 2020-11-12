use std::{
    env,
    fs::{File, OpenOptions},
    io,
    os::{raw::c_int, unix::process::CommandExt},
    process::{self, Command, Stdio},
};

use coreutils_core::{
    libc::{signal, ENOENT, SIGHUP, SIG_IGN},
    os::tty::IsTTY,
};

mod cli;

fn main() {
    let matches = cli::create_app().get_matches();

    // Ok to unwrap: COMMAND is required
    let mut cmd = matches.values_of("COMMAND").unwrap();
    // Ok to unwrap: Since COMMAND is required, there must be the first value
    let command_name = cmd.next().unwrap();
    let args: Vec<_> = cmd.collect();

    let mut command = Command::new(command_name);
    let mut command_c = command.args(&args);

    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create(true).append(true);

    // If standard input is a terminal, redirect it from an unreadable file.
    if io::stdin().is_tty() {
        command_c = command_c.stdin(Stdio::null());
    }

    if io::stdout().is_tty() {
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
    if io::stderr().is_tty() {
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

    if let Some(ENOENT) = err.raw_os_error() {
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
