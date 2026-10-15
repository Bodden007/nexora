use super::adapter_names;
use std::io::{self, Write};

enum Selection {
    Keep,
    Clear,
    Index(usize),
}

fn select_index(count: usize, has_current: bool) -> io::Result<Selection> {
    loop {
        if has_current {
            print!("Номер (Enter — оставить, 0 — сбросить): ");
        } else {
            print!("Номер (0 — пропустить): ");
        }

        io::stdout().flush()?;

        let mut input = String::new();

        if io::stdin().read_line(&mut input)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Ввод завершён",
            ));
        }

        let value = input.trim();

        if value.is_empty() && has_current {
            return Ok(Selection::Keep);
        }

        match value.parse::<usize>() {
            Ok(0) => return Ok(Selection::Clear),
            Ok(number) if number <= count => {
                return Ok(Selection::Index(number - 1));
            }
            _ => println!("Некорректный номер."),
        }
    }
}

pub fn select_serial(current: Option<&str>) -> io::Result<Option<String>> {
    println!("\nПоследовательные порты:");

    let ports = serialport::available_ports()?;

    for (index, port) in ports.iter().enumerate() {
        println!("{}. {}", index + 1, port.port_name);
    }

    if ports.is_empty() {
        println!("Порты не обнаружены");
    }

    if let Some(port) = current {
        println!("Сохранён: {port}");
    }

    match select_index(ports.len(), current.is_some())? {
        Selection::Keep => Ok(current.map(str::to_owned)),
        Selection::Clear => Ok(None),
        Selection::Index(index) => Ok(Some(ports[index].port_name.clone())),
    }
}

pub fn select_ethernet(current: Option<&str>) -> io::Result<Option<String>> {
    println!("\nEthernet-адаптеры (CompactRIO):");

    let devices = pcap::Device::list().map_err(|error| io::Error::other(error.to_string()))?;

    let names = adapter_names::get_names();

    for (index, device) in devices.iter().enumerate() {
        let name =
            adapter_names::get_name(&device.name, &names).unwrap_or_else(|| device.name.clone());

        println!(
            "{}. {} | {} | UP: {}",
            index + 1,
            name,
            device.name,
            device.flags.is_up()
        );
    }

    if devices.is_empty() {
        println!("Адаптеры не обнаружены");
    }

    if let Some(adapter) = current {
        println!("Сохранён: {adapter}");
    }

    match select_index(devices.len(), current.is_some())? {
        Selection::Keep => Ok(current.map(str::to_owned)),
        Selection::Clear => Ok(None),
        Selection::Index(index) => Ok(Some(devices[index].name.clone())),
    }
}

pub fn select_modbus(current: Option<&str>) -> io::Result<Option<String>> {
    println!("\nВыход Modbus TCP:");

    let devices = pcap::Device::list().map_err(|error| io::Error::other(error.to_string()))?;

    let names = adapter_names::get_names();

    println!("1. Localhost | 127.0.0.1");

    for (index, device) in devices.iter().enumerate() {
        let name =
            adapter_names::get_name(&device.name, &names).unwrap_or_else(|| device.name.clone());

        println!(
            "{}. {} | {} | UP: {}",
            index + 2,
            name,
            device.name,
            device.flags.is_up()
        );
    }

    if let Some(adapter) = current {
        println!("Сохранён: {adapter}");
    }

    match select_index(devices.len() + 1, current.is_some())? {
        Selection::Keep => Ok(current.map(str::to_owned)),
        Selection::Clear => Ok(None),
        Selection::Index(0) => Ok(Some("localhost".to_string())),
        Selection::Index(index) => Ok(Some(devices[index - 1].name.clone())),
    }
}
