use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    #[cfg(not(target_os = "openbsd"))]
    let yaml = load_yaml!("src/who.yml");
    #[cfg(target_os = "openbsd")]
    let yaml = load_yaml!("src/who_openbsd.yml");

    let mut app = App::from_yaml(yaml)
        .help_message("Display help information")
        .version_message("Display version information");

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("who", Shell::Zsh, out_dir.clone());
    app.gen_completions("who", Shell::Fish, out_dir.clone());
    app.gen_completions("who", Shell::Bash, out_dir.clone());
    app.gen_completions("who", Shell::PowerShell, out_dir.clone());
    app.gen_completions("who", Shell::Elvish, out_dir);
}
