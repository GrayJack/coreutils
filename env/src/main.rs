use std::{
    collections::HashMap,
    env,
    os::unix::process::CommandExt,
    process::{self, Command},
};

mod cli;

const ENOENT: i32 = 2;

fn main() {
    let matches = cli::create_app().get_matches();

    let mut kv = HashMap::new();
    let mut cmd = Vec::new();

    if let Some(m) = matches.values_of("OPTIONS") {
        for word in m {
            if let Some(index) = word.find('=') {
                let (k, v) = word.split_at(index);
                kv.insert(k.to_owned(), v.get(1..).unwrap_or("").to_owned());
            } else {
                cmd.push(word);
            }
        }
    };

    let unset_keys: Vec<_> =
        matches.values_of("unset").map(|values| values.collect()).unwrap_or_default();

    let ignore_environemnt = matches.is_present("ignore_environment");
    let eol = if matches.is_present("null") { '\0' } else { '\n' };

    let env_vars = {
        let mut env_vars = HashMap::new();

        if !ignore_environemnt {
            for (key, value) in env::vars() {
                if !unset_keys.contains(&key.as_str()) {
                    env_vars.insert(key, value);
                }
            }
        }

        for (key, value) in kv {
            env_vars.insert(key, value);
        }

        env_vars
    };

    if cmd.is_empty() {
        for (key, value) in env_vars {
            print!("{}={}{}", key, value, eol);
        }

        print!("{}", eol);
    } else {
        let command = cmd.remove(0);
        let args = cmd;
        println!("{} ", command);

        let err = Command::new(command).args(args).env_clear().envs(&env_vars).exec();

        if err.raw_os_error().unwrap() == ENOENT {
            eprintln!("env: '{}': {}", command, err);
            process::exit(127);
        } else {
            eprintln!("env: {}", err);
            process::exit(126);
        }
    }
}
