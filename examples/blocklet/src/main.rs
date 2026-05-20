use std::{env, fs, io::Read as _, process::ExitCode};

fn main() -> ExitCode {
    let arg = env::args().nth(1);
    let source = match arg {
        Some(path) if path != "-" => match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("blocklet: cannot read {path}: {e}");
                return ExitCode::from(2);
            }
        },
        _ => {
            let mut s = String::new();
            if std::io::stdin().read_to_string(&mut s).is_err() {
                eprintln!("blocklet: failed to read stdin");
                return ExitCode::from(2);
            }
            s
        }
    };

    match blocklet::eval(&source) {
        Ok(v) => {
            println!("{v}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("blocklet: {e}");
            ExitCode::FAILURE
        }
    }
}
