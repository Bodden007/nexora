use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;

use super::{capture, serial};

const NONE: u8 = 0;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Protocol {
    CompactRio = 1,
    Flecs = 2,
}

pub fn select_source(selected: &AtomicU8, protocol: Protocol) -> bool {
    selected
        .compare_exchange(NONE, protocol as u8, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

pub fn run(ethernet: &str, serial_port: &str) -> Result<(), String> {
    let selected = Arc::new(AtomicU8::new(NONE));
    let mut workers = Vec::new();

    if !ethernet.trim().is_empty() {
        let interface = ethernet.to_owned();
        let state = Arc::clone(&selected);

        workers.push(thread::spawn(move || capture::run(&interface, &state)));
    }

    if !serial_port.trim().is_empty() {
        let port = serial_port.to_owned();
        let state = Arc::clone(&selected);

        workers.push(thread::spawn(move || serial::run(&port, &state)));
    }

    if workers.is_empty() {
        return Err("Источники данных не настроены".into());
    }

    println!("[NEXORA] Обнаружение источников запущено");

    let mut errors = Vec::new();

    for worker in workers {
        match worker.join() {
            Ok(Ok(())) => {}

            Ok(Err(error)) => {
                eprintln!("[NEXORA] {error}");
                errors.push(error);
            }

            Err(_) => {
                errors.push("Поток обнаружения завершился аварийно".to_string());
            }
        }
    }

    match selected.load(Ordering::Acquire) {
        value if value == Protocol::CompactRio as u8 => {
            println!("[NEXORA] Источник: CompactRIO LLC");
        }

        value if value == Protocol::Flecs as u8 => {
            println!("[NEXORA] Источник: FLECS ASCII");
        }

        _ => {
            return Err(if errors.is_empty() {
                "Источник данных не обнаружен".to_string()
            } else {
                errors.join("; ")
            });
        }
    }

    println!("[NEXORA] Источник зафиксирован");

    Ok(())
}
