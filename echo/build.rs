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

    app.gen_completions("echo", Shell::Zsh, out_dir.clone());
    app.gen_completions("echo", Shell::Fish, out_dir.clone());
    app.gen_completions("echo", Shell::Bash, out_dir.clone());
    app.gen_completions("echo", Shell::PowerShell, out_dir.clone());
    app.gen_completions("echo", Shell::Elvish, out_dir);
}
