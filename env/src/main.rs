use std::collections::HashMap;
use std::process;
use std::process::Command;
use std::{env, io};

use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("env.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let mut kv = HashMap::new();
    let mut cmd = Vec::new();
    matches.values_of("OPTIONS").map(|m| {
        for word in m {
            let word_str: String = word.to_owned();
            match word_str.find('=') {
                Some(index) => {
                    let (k, v) = word_str.split_at(index);
                    kv.insert(k.to_owned(), v.get(1..).unwrap_or("").to_owned());
                }
                None => {
                    cmd.push(word_str);
                }
            }
        }
    });
    let mut unset_keys = Vec::new();
    matches.values_of("REMOVE_VAR").map(|keys| {
        keys.for_each(|k| unset_keys.push(k.to_owned()));
    });
    match env(
        kv,
        matches.is_present("IGNORE_ENVIRONMENT"),
        matches.is_present("NULL_EOL"),
        cmd,
        unset_keys,
    ) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("echo: Failed to write to stdout.\n{}", e);
            process::exit(1);
        }
    };
}

// run `man env`
fn env(
    kv: HashMap<String, String>,
    ignore_environemnt: bool,
    null_eol: bool,
    mut cmd: Vec<String>,
    unset_keys: Vec<String>,
) -> io::Result<()> {
    // Prints each argument on a separate line
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

    if cmd.len() > 0 {
        println!("Spawning program");
        let cmd_name = cmd.remove(0);
        println!("{} ", cmd_name);
        Command::new(cmd_name.clone())
            .env_clear()
            .args(cmd)
            .envs(&env_vars)
            .status()
            .expect(&format!("{} failed to start.", cmd_name));
        println!("done spawining..");
    } else {
        for (key, value) in env_vars.iter() {
            print!("{}={}", key, value);
            if !null_eol {
                print!("\n");
            }
        }

        if !null_eol {
            // Let's be polite, and let the shell prompt start on a new line
            println!("")
        }
    }

    Ok(())
}
