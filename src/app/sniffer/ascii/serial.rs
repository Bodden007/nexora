use std::io::ErrorKind;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

use super::super::coordinator::{Protocol, select_source};
use super::super::detection::is_flecs_line;
use super::flecs;

const BAUD_RATE: u32 = 9600;
const TIMEOUT_MS: u64 = 100;
const REQUIRED_MESSAGES: usize = 3;
const MAX_LINE_LENGTH: usize = 4096;

const DATA_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_INTERVAL: Duration = Duration::from_secs(1);

fn should_run(selected: &AtomicU8) -> bool {
    let state = selected.load(Ordering::Acquire);

    state == 0 || state == Protocol::Flecs as u8
}

pub fn run(port_name: &str, selected: &AtomicU8) -> Result<(), String> {
    let mut last_error = String::new();

    while should_run(selected) {
        let result = match serialport::new(port_name, BAUD_RATE)
            .timeout(Duration::from_millis(TIMEOUT_MS))
            .open()
        {
            Ok(mut port) => {
                println!("[FLECS] Порт открыт: {port_name}");

                let result = read_flecs(port.as_mut(), port_name, selected);

                drop(port);
                result
            }

            Err(error) => Err(format!("Не удалось открыть {port_name}: {error}")),
        };

        if !should_run(selected) {
            break;
        }

        match result {
            Ok(()) => {
                last_error.clear();
            }

            Err(error) => {
                if error != last_error {
                    eprintln!("[FLECS] {error}");
                    last_error = error;
                }
            }
        }

        wait_retry(selected);
    }

    Ok(())
}

fn wait_retry(selected: &AtomicU8) {
    let start = Instant::now();

    while should_run(selected) && start.elapsed() < RETRY_INTERVAL {
        std::thread::sleep(Duration::from_millis(100));
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

    let mut last_valid_message = Instant::now();

    while should_run(selected) {
        match port.read(&mut input) {
            Ok(count) => {
                for &byte in &input[..count] {
                    if !should_run(selected) {
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

                        let parsed = std::str::from_utf8(&line).ok();
                        let valid = parsed.is_some_and(is_flecs_line);

                        if !valid {
                            valid_messages = 0;
                            line.clear();
                            continue;
                        }

                        last_valid_message = Instant::now();

                        if selected.load(Ordering::Acquire) == 0 {
                            valid_messages += 1;

                            if valid_messages >= REQUIRED_MESSAGES
                                && select_source(selected, Protocol::Flecs)
                            {
                                println!("[FLECS] Протокол подтверждён");
                                println!("[FLECS] Порт: {port_name}");
                                println!(
                                    "[FLECS] Корректных сообщений: \
                                        {valid_messages}"
                                );
                            }
                        }

                        if selected.load(Ordering::Acquire) == Protocol::Flecs as u8
                            && let Some(text) = parsed
                        {
                            match flecs::parse_line(text) {
                                Ok(data) => {
                                    println!("[FLECS] {data:#?}");
                                }
                                Err(error) => {
                                    eprintln!("[FLECS] Ошибка парсера: {error}");
                                }
                            }
                        }

                        line.clear();
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

        if last_valid_message.elapsed() >= DATA_TIMEOUT {
            return Err(format!(
                "Нет корректных данных от {port_name} \
                 в течение 5 секунд"
            ));
        }
    }

    Ok(())
}
