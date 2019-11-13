use std::{
    collections::HashMap,
    env, io,
    process::{self, Command},
};

use clap::{load_yaml, App, AppSettings::ColoredHelp};

fn main() {
    let yaml = load_yaml!("env.yml");
    let matches = App::from_yaml(yaml)
        .settings(&[ColoredHelp])
        .help_message("Display help information")
        .version_message("Display version information")
        .get_matches();

    let mut kv = HashMap::new();
    let mut cmd = Vec::new();

    if let Some(m) = matches.values_of("OPTIONS") {
        for word in m {
            let word_str: String = word.to_owned();
            if let Some(index) = word_str.find('=') {
                let (k, v) = word_str.split_at(index);
                kv.insert(k.to_owned(), v.get(1..).unwrap_or("").to_owned());
            } else {
                cmd.push(word_str);
            }
        }
    };

    let mut unset_keys = Vec::new();

    if let Some(keys) = matches.values_of("unset") {
        keys.for_each(|k| unset_keys.push(k.to_owned()));
    };

    match env(
        kv,
        matches.is_present("ignore_environment"),
        matches.is_present("null"),
        cmd,
        &unset_keys,
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("echo: Failed to write to stdout.\n{}", e);
            process::exit(1);
        },
    }
}

// run `man env`
fn env(
    kv: HashMap<String, String>, ignore_environemnt: bool, null_eol: bool, mut cmd: Vec<String>,
    unset_keys: &[String],
) -> io::Result<()>
{
    let mut env_vars = HashMap::new();

    for (key, value) in env::vars() {
        if !unset_keys.contains(&key) {
            env_vars.insert(key, value);
        }
    }

    if ignore_environemnt {
        env_vars = HashMap::new();
    }

    for (key, value) in kv {
        env_vars.insert(key, value);
    }

    if cmd.is_empty() {
        for (key, value) in env_vars {
            print!("{}={}", key, value);
            if !null_eol {
                println!();
            }
        }

        if !null_eol {
            // Let's be polite, and let the shell prompt start on a new line
            println!()
        }
    } else {
        let cmd_name = cmd.remove(0);
        println!("{} ", cmd_name);

        Command::new(cmd_name.clone())
            .env_clear()
            .args(cmd)
            .envs(&env_vars)
            .status()
            .unwrap_or_else(|_| panic!("{} failed to start.", cmd_name));
    }

    Ok(())
}
