use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    let yaml = load_yaml!("src/rmdir.yml");
    let mut app = App::from_yaml(yaml)
        .help_message("Display help information")
        .version_message("Display version information");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("rmdir", Shell::Zsh, out_dir.clone());
    app.gen_completions("rmdir", Shell::Fish, out_dir.clone());
    app.gen_completions("rmdir", Shell::Bash, out_dir.clone());
    app.gen_completions("rmdir", Shell::PowerShell, out_dir.clone());
    app.gen_completions("rmdir", Shell::Elvish, out_dir);
}
