use std::{env, io, process::ExitCode};

fn main() -> ExitCode {
    let result = match env::args().nth(1).as_deref() {
        Some("--conf") => nexora::configure(),
        Some(arg) => Err(format!("Неизвестный аргумент: {arg}")),
        None => nexora::run(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("NEXORA: {error}");
            eprintln!("\nНажмите Enter для завершения...");

            let mut input = String::new();
            let _ = io::stdin().read_line(&mut input);

            ExitCode::FAILURE
        }
    }
}
