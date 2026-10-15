pub mod app;

use app::{config, sniffer};

pub fn run() -> Result<(), String> {
    let configuration = config::load()?;

    println!("NEXORA: конфигурация загружена");
    println!("FLECS: {}", configuration.serial.port);
    println!("CompactRIO: {}", configuration.ethernet.interface);
    println!("Modbus TCP: {}", configuration.modbus.interface);

    if !configuration.ethernet.interface.is_empty() {
        sniffer::run(&configuration.ethernet.interface)?;
    }

    Ok(())
}

pub fn configure() -> Result<(), String> {
    config::run()
}
