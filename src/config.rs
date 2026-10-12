mod adapter_names;
mod interfaces;

use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
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
    let serial = config.serial.port.trim();
    let ethernet = config.ethernet.interface.trim();
    let modbus = config.modbus.interface.trim();

    if serial.is_empty() && ethernet.is_empty() {
        return Err("Не настроен ни один источник данных".to_string());
    }

    if modbus.is_empty() {
        return Err("Не настроен выход Modbus TCP".to_string());
    }

    if !ethernet.is_empty() && ethernet == modbus {
        return Err("Вход LLC и выход Modbus TCP используют один адаптер".to_string());
    }

    Ok(())
}

fn read_config(path: &Path) -> Result<Configuration, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Ошибка чтения {}: {error}", path.display()))?;

    toml::from_str(&content).map_err(|error| format!("Ошибка TOML: {error}"))
}

pub fn load() -> Result<Configuration, String> {
    let path = config_path()?;
    let config = read_config(&path)?;

    validate(&config)?;
    Ok(config)
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    unsafe extern "system" {
        fn ReplaceFileW(
            replaced: *const u16,
            replacement: *const u16,
            backup: *const u16,
            flags: u32,
            exclude: *const core::ffi::c_void,
            reserved: *const core::ffi::c_void,
        ) -> i32;
    }

    let destination_wide: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();

    let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();

    if destination.exists() {
        // SAFETY: Пути завершаются нулём, указатели действительны
        // на протяжении вызова Windows API.
        let result = unsafe {
            ReplaceFileW(
                destination_wide.as_ptr(),
                source_wide.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        };

        if result == 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    } else {
        fs::rename(source, destination)
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

fn save(config: &Configuration) -> Result<(), String> {
    validate(config)?;

    let path = config_path()?;
    let content = toml::to_string_pretty(config)
        .map_err(|error| format!("Ошибка сериализации TOML: {error}"))?;

    let directory = path.parent().ok_or("Каталог конфигурации не найден")?;

    let (temporary, mut file) = {
        let mut created = None;

        for index in 0..100 {
            let name = format!(".nexora-{}-{index}.tmp", std::process::id());

            let temporary = directory.join(name);

            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => {
                    created = Some((temporary, file));
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(format!("Ошибка временного файла: {error}")),
            }
        }

        created.ok_or("Не удалось создать временный файл")?
    };

    let result = (|| -> io::Result<()> {
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);

        replace_file(&temporary, &path)
    })();

    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!("Ошибка сохранения конфигурации: {error}"));
    }

    Ok(())
}

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

    let path = config_path()?;

    let mut config = if path.exists() {
        match read_config(&path) {
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

    save(&config)?;

    println!("Конфигурация сохранена: {}", path.display());
    Ok(())
}

pub fn run() -> Result<(), String> {
    configure()
}
