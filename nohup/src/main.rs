extern crate signal_hook;

use clap::{load_yaml, App};
use coreutils_core::{file_descriptor::FileDescriptor, tty::isatty};
use std::{
    env,
    fs::{File, OpenOptions},
    io::Error,
    process::{Command, Stdio},
};

fn main() -> Result<(), Error> {
    let yaml = load_yaml!("nohup.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let args_val = matches.values_of("COMMAND").unwrap();
    let mut args = args_val.into_iter().map(|x| x.to_owned()).collect::<Vec<String>>();
    let command_name = args.remove(0);

    let mut command = &mut Command::new(command_name);
    // If standard input is a terminal, redirect it from an unreadable file.
    if isatty(FileDescriptor::StdIn) {
        command = command.stdin(Stdio::null());
    }
    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create(true).append(true);
    let get_stdout = || -> File {
        open_opts.open("nohup.out").unwrap_or_else(|_| {
            for (key, value) in env::vars() {
                if key == "HOME" {
                    return open_opts.open(value + "nohup.out").unwrap();
                }
            }
            panic!("There is no $HOME variable in the environment.");
        })
    };
    let _signal_handler = unsafe {
        signal_hook::register(signal_hook::SIGINT, || {
            // ignore
            println!("Hello world! :D ");
        })
    }?;
    if isatty(FileDescriptor::StdOut) {
        // Try to open in write append nohup.out else open $HOME/nohup.out
        command = command.stdout(get_stdout());
    }
    // If standard error is a terminal, redirect it to standard output.
    if isatty(FileDescriptor::StdErr) {
        command = command.stderr(get_stdout());
    }
    command.status().expect("Error while invoking the command.");
    Ok(())
}
