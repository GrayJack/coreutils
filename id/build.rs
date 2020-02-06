use std::env;

use clap::{load_yaml, App, Shell};

fn main() {
    #[cfg(any(target_os = "freebsd", target_os = "macos"))]
    let yaml = load_yaml!("src/id_audit.yml");
    #[cfg(any(target_os = "openbsd"))]
    let yaml = load_yaml!("src/id_rtable.yml");
    #[cfg(not(any(target_os = "freebsd", target_os = "macos", target_os = "openbsd")))]
    let yaml = load_yaml!("src/id.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return,
    };

    app.gen_completions("id", Shell::Zsh, out_dir.clone());
    app.gen_completions("id", Shell::Fish, out_dir.clone());
    app.gen_completions("id", Shell::Bash, out_dir.clone());
    app.gen_completions("id", Shell::PowerShell, out_dir.clone());
    app.gen_completions("id", Shell::Elvish, out_dir);
}
