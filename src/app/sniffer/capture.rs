use pcap::{Active, Capture};

pub fn run(interface: &str) -> Result<(), String> {
    if interface.trim().is_empty() {
        return Err("Ethernet-адаптер CompactRIO не настроен".to_string());
    }

    let _capture: Capture<Active> = Capture::from_device(interface)
        .map_err(|error| format!("Адаптер {interface}: {error}"))?
        .promisc(true)
        .timeout(1000)
        .open()
        .map_err(|error| format!("Ошибка открытия {interface}: {error}"))?;

    println!("LLC Sniffer: адаптер успешно открыт");
    println!("Интерфейс: {interface}");

    Ok(())
}
