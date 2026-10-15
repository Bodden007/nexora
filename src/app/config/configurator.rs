use std::io::{self, Write};

use super::{interfaces, model::Configuration, storage};

fn confirm(message: &str) -> io::Result<bool> {
    print!("{message} [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();

    if io::stdin().read_line(&mut input)? == 0 {
        return Ok(false);
    }

    Ok(matches!(
        input.trim().to_lowercase().as_str(),
        "y" | "yes" | "д" | "да"
    ))
}

fn configure() -> Result<(), String> {
    println!("NEXORA — конфигуратор");

    let path = storage::config_path()?;

    let mut config = if path.exists() {
        match storage::read_config(&path) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("\n{error}");

                let accepted =
                    confirm("\nСоздать новую конфигурацию?").map_err(|error| error.to_string())?;

                if !accepted {
                    println!("Конфигурация не изменена");
                    return Ok(());
                }

                Configuration::default()
            }
        }
    } else {
        Configuration::default()
    };

    config.serial.port =
        interfaces::select_serial(Some(config.serial.port.as_str()).filter(|v| !v.is_empty()))
            .map_err(|error| error.to_string())?
            .unwrap_or_default();

    config.ethernet.interface = interfaces::select_ethernet(
        Some(config.ethernet.interface.as_str()).filter(|v| !v.is_empty()),
    )
    .map_err(|error| error.to_string())?
    .unwrap_or_default();

    config.modbus.interface =
        interfaces::select_modbus(Some(config.modbus.interface.as_str()).filter(|v| !v.is_empty()))
            .map_err(|error| error.to_string())?
            .unwrap_or_default();

    println!("\nИтоговая конфигурация:");
    println!("FLECS: {}", config.serial.port);
    println!("CompactRIO: {}", config.ethernet.interface);
    println!("Modbus TCP: {}", config.modbus.interface);

    if !confirm("\nСохранить конфигурацию?").map_err(|error| error.to_string())?
    {
        println!("Сохранение отменено");
        return Ok(());
    }

    storage::save(&config)?;

    println!("Конфигурация сохранена: {}", path.display());
    Ok(())
}

pub fn run() -> Result<(), String> {
    configure()
}
