use std::env;

use clap::Shell;

#[path = "src/cli.rs"]
mod cli;

fn main() {
    let mut app = cli::create_app();

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("No OUT_DIR: {}", err);
            return;
        },
    };

    app.gen_completions("printf", Shell::Zsh, out_dir.clone());
    app.gen_completions("printf", Shell::Fish, out_dir.clone());
    app.gen_completions("printf", Shell::Bash, out_dir.clone());
    app.gen_completions("printf", Shell::PowerShell, out_dir.clone());
    app.gen_completions("printf", Shell::Elvish, out_dir);
}
