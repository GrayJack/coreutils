use std::env;
use std::fs;

fn log<S: Into<String>>(msg: S) {
    println!("mkdir: {}", msg.into());
}


fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        log("missing operand\nTry 'mkdir --help' for more information.");
    }

    if args.len() == 2 {
        let dir_path = &args[1];
        match fs::create_dir(dir_path) {
            Ok(_) => (),
            Err(e) => {
                log(format!("cannot create directory '{}': {}", dir_path, e))
            }
        }
    }
}
