use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{Duration, Instant};

use pcap::{Active, Capture, Error};

use super::coordinator::{Protocol, select_source};
use super::detector::is_crio_frame;

const REQUIRED_FRAMES: usize = 3;
const RETRY_INTERVAL: Duration = Duration::from_secs(1);
const RETRY_CHECK_INTERVAL: Duration = Duration::from_millis(100);
const CAPTURE_TIMEOUT_MS: i32 = 1000;

pub fn run(interface: &str, selected: &AtomicU8) -> Result<(), String> {
    let mut last_error = String::new();

    while selected.load(Ordering::Acquire) == 0 {
        let result = match Capture::from_device(interface) {
            Ok(device) => {
                match device.promisc(true).timeout(CAPTURE_TIMEOUT_MS).open() {
                    Ok(mut capture) => {
                        println!("[LLC] Адаптер открыт: {interface}");

                        let result = read_llc(&mut capture, interface, selected);

                        // Закрываем pcap-дескриптор до повтора.
                        drop(capture);

                        result
                    }

                    Err(error) => Err(format!("Ошибка открытия {interface}: {error}")),
                }
            }

            Err(error) => Err(format!("Адаптер {interface}: {error}")),
        };

        if selected.load(Ordering::Acquire) != 0 {
            break;
        }

        match result {
            Ok(()) => {}

            Err(error) => {
                if error != last_error {
                    eprintln!("[LLC] {error}");
                    last_error = error;
                }
            }
        }

        wait_retry(selected);
    }

    Ok(())
}

fn read_llc(
    capture: &mut Capture<Active>,
    interface: &str,
    selected: &AtomicU8,
) -> Result<(), String> {
    let mut valid_frames = 0usize;

    println!("[LLC] Ожидание CompactRIO...");

    while selected.load(Ordering::Acquire) == 0 {
        match capture.next_packet() {
            Ok(packet) => {
                if !is_crio_frame(packet.data) {
                    continue;
                }

                valid_frames += 1;

                if valid_frames < REQUIRED_FRAMES {
                    continue;
                }

                if select_source(selected, Protocol::CompactRio) {
                    println!("[LLC] CompactRIO подтверждён");
                    println!("[LLC] Интерфейс: {interface}");
                    println!("[LLC] Корректных кадров: {valid_frames}");

                    if let Some(mac) = packet.data.get(6..12) {
                        println!(
                            "[LLC] Source MAC: \
                             {:02X}:{:02X}:{:02X}:\
                             {:02X}:{:02X}:{:02X}",
                            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                        );
                    }
                }

                return Ok(());
            }

            Err(Error::TimeoutExpired) => {
                // Отсутствие пакетов — нормальное состояние.
            }

            Err(error) => {
                return Err(format!("Ошибка захвата на {interface}: {error}"));
            }
        }
    }

    Ok(())
}

fn wait_retry(selected: &AtomicU8) {
    let start = Instant::now();

    while start.elapsed() < RETRY_INTERVAL {
        if selected.load(Ordering::Acquire) != 0 {
            return;
        }

        let remaining = RETRY_INTERVAL.saturating_sub(start.elapsed());

        std::thread::sleep(remaining.min(RETRY_CHECK_INTERVAL));
    }
}
