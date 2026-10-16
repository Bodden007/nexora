use std::io::ErrorKind;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use super::coordinator::{Protocol, select_source};
use super::detector::is_flecs_line;

const BAUD_RATE: u32 = 9600;
const TIMEOUT_MS: u64 = 100;
const REQUIRED_MESSAGES: usize = 3;
const MAX_LINE_LENGTH: usize = 4096;

pub fn run(port_name: &str, selected: &AtomicU8) -> Result<(), String> {
    let mut last_error = String::new();

    while selected.load(Ordering::Acquire) == 0 {
        let result = match serialport::new(port_name, BAUD_RATE)
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .open()
        {
            Ok(mut port) => {
                println!("[FLECS] Порт открыт: {port_name}");

                let result = read_flecs(port.as_mut(), port_name, selected);

                // Закрываем дескриптор до повторного открытия.
                drop(port);

                result
            }

            Err(error) => Err(format!("Не удалось открыть {port_name}: {error}")),
        };

        if selected.load(Ordering::Acquire) != 0 {
            break;
        }

        if let Err(error) = result {
            // Не засоряем консоль одинаковыми ошибками.
            if error != last_error {
                eprintln!("[FLECS] {error}");
                last_error = error;
            }
        }

        wait_retry(selected);
    }

    Ok(())
}

fn wait_retry(selected: &AtomicU8) {
    const RETRY_INTERVAL: Duration = Duration::from_secs(1);
    const CHECK_INTERVAL: Duration = Duration::from_millis(100);

    let mut elapsed = Duration::ZERO;

    while elapsed < RETRY_INTERVAL {
        if selected.load(Ordering::Acquire) != 0 {
            return;
        }

        std::thread::sleep(CHECK_INTERVAL);
        elapsed += CHECK_INTERVAL;
    }
}

fn read_flecs(
    port: &mut dyn serialport::SerialPort,
    port_name: &str,
    selected: &AtomicU8,
) -> Result<(), String> {
    let mut input = [0u8; 1024];
    let mut line = Vec::<u8>::new();
    let mut valid_messages = 0usize;
    let mut skip_lf = false;
    let mut discard_line = false;

    while selected.load(Ordering::Acquire) == 0 {
        match port.read(&mut input) {
            Ok(count) => {
                for &byte in &input[..count] {
                    if selected.load(Ordering::Acquire) != 0 {
                        return Ok(());
                    }

                    if skip_lf {
                        skip_lf = false;
                        if byte == b'\n' {
                            continue;
                        }
                    }

                    if byte == b'\r' || byte == b'\n' {
                        if byte == b'\r' {
                            skip_lf = true;
                        }

                        if discard_line {
                            discard_line = false;
                            line.clear();
                            valid_messages = 0;
                            continue;
                        }

                        if line.is_empty() {
                            continue;
                        }

                        let valid = std::str::from_utf8(&line).is_ok_and(is_flecs_line);

                        line.clear();

                        if valid {
                            valid_messages += 1;
                        } else {
                            valid_messages = 0;
                        }

                        if valid_messages >= REQUIRED_MESSAGES {
                            if select_source(selected, Protocol::Flecs) {
                                println!("[FLECS] Протокол подтверждён");
                                println!("[FLECS] Порт: {port_name}");
                                println!(
                                    "[FLECS] Корректных сообщений: \
                                     {valid_messages}"
                                );
                            }

                            return Ok(());
                        }

                        continue;
                    }

                    if discard_line {
                        continue;
                    }

                    if line.len() >= MAX_LINE_LENGTH {
                        line.clear();
                        discard_line = true;
                        valid_messages = 0;
                        continue;
                    }

                    line.push(byte);
                }
            }

            Err(error) if error.kind() == ErrorKind::TimedOut => {}

            Err(error) => {
                return Err(format!("Ошибка чтения {port_name}: {error}"));
            }
        }
    }

    Ok(())
}
