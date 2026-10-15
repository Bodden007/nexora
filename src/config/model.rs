use serde::{Deserialize, Serialize};

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

pub(super) fn validate(config: &Configuration) -> Result<(), String> {
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
