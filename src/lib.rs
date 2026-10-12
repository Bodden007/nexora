pub mod config;

pub fn run() -> Result<(), String> {
    let configuration = config::load()?;

    println!("NEXORA: конфигурация загружена");
    println!("FLECS: {}", configuration.serial.port);
    println!("CompactRIO: {}", configuration.ethernet.interface);
    println!("Modbus TCP: {}", configuration.modbus.interface);

    println!("NEXORA: штатный запуск пока не реализован");

    Ok(())
}

pub fn configure() -> Result<(), String> {
    config::run()
}
