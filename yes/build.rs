use std::env;

use clap::{ App, Shell, load_yaml };

fn main() {
    let yaml = load_yaml!("src/yes.yml");
    let mut app = App::from_yaml(yaml);

    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        _ => return
    };

    app.gen_completions("yes", Shell::Zsh, out_dir.clone());
    app.gen_completions("yes", Shell::Fish, out_dir.clone());
    app.gen_completions("yes", Shell::Bash, out_dir.clone());
    app.gen_completions("yes", Shell::PowerShell, out_dir.clone());
    app.gen_completions("yes", Shell::Elvish, out_dir);
}
