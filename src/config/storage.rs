use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use super::model::{Configuration, validate};

pub(super) fn config_path() -> Result<PathBuf, String> {
    let executable = env::current_exe().map_err(|error| format!("Путь приложения: {error}"))?;

    Ok(executable
        .parent()
        .ok_or("Каталог приложения не найден")?
        .join("nexora.toml"))
}

pub(super) fn read_config(path: &Path) -> Result<Configuration, String> {
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

pub(super) fn save(config: &Configuration) -> Result<(), String> {
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
