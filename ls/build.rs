use std::{env, fs::File, io::BufWriter};

use clap::crate_name;
use clap_generate::{Generator, Shell};

#[path = "src/cli.rs"]
mod cli;

fn main() {
    let mut app = cli::create_app();
    app.set_bin_name(crate_name!());

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("No OUT_DIR: {}", err);
            return;
        },
    };


    let shells = [Shell::Bash, Shell::Elvish, Shell::Fish, Shell::PowerShell, Shell::Zsh];

    for shell in shells {
        let file_name = format!("{}/{}", out_dir, shell.file_name(app.get_name()));
        let mut file = BufWriter::new(
            File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(file_name)
                .expect("Unable to open file"),
        );

        shell.generate(&app, &mut file)
    }
}
