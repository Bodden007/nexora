use std::{env, io, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args.as_slice() {
        [] => nexora::run(),
        [arg] if arg == "--conf" => nexora::configure(),
        _ => Err(format!("Некорректные аргументы: {}", args.join(" "))),
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
