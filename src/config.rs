mod adapter_names;
mod interfaces;

use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub serial: SerialConfig,
    pub ethernet: EthernetConfig,
    pub modbus: ModbusConfig,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SerialConfig {
    pub port: String,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EthernetConfig {
    pub interface: String,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModbusConfig {
    pub interface: String,
}

fn config_path() -> Result<PathBuf, String> {
    let executable = env::current_exe().map_err(|error| format!("Путь приложения: {error}"))?;

    Ok(executable
        .parent()
        .ok_or("Каталог приложения не найден")?
        .join("nexora.toml"))
}

fn validate(config: &Configuration) -> Result<(), String> {
    if config.serial.port.trim().is_empty()
        || config.ethernet.interface.trim().is_empty()
        || config.modbus.interface.trim().is_empty()
    {
        return Err("Не все интерфейсы настроены".to_string());
    }

    if config.ethernet.interface == config.modbus.interface {
        return Err("Вход LLC и выход Modbus TCP используют один адаптер".to_string());
    }

    Ok(())
}

pub fn load() -> Result<Configuration, String> {
    let path = config_path()?;

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Не удалось прочитать {}: {error}", path.display()))?;

    let config: Configuration =
        toml::from_str(&content).map_err(|error| format!("Ошибка TOML: {error}"))?;

    validate(&config)?;

    Ok(config)
}

fn save(config: &Configuration) -> Result<(), String> {
    validate(config)?;

    let path = config_path()?;

    let content = toml::to_string_pretty(config).map_err(|error| error.to_string())?;

    fs::write(&path, content)
        .map_err(|error| format!("Не удалось записать {}: {error}", path.display()))
}

fn confirm() -> io::Result<bool> {
    print!("\nСохранить конфигурацию? [y/N]: ");
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
    let mut config = match config_path()? {
        path if path.exists() => {
            let content =
                fs::read_to_string(&path).map_err(|error| format!("Ошибка чтения: {error}"))?;

            toml::from_str::<Configuration>(&content)
                .map_err(|error| format!("Ошибка TOML: {error}"))?
        }
        _ => Configuration::default(),
    };

    println!("NEXORA — конфигуратор");

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

    validate(&config)?;

    if !confirm().map_err(|error| error.to_string())? {
        println!("Сохранение отменено");
        return Ok(());
    }

    save(&config)?;
    println!("Конфигурация сохранена: {}", config_path()?.display());

    Ok(())
}

pub fn run() -> Result<(), String> {
    configure()
}
