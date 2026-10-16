const CRIO_SIGNATURE: [u8; 9] = [0xD0, 0x0D, 0x00, 0xA8, 0x6F, 0x46, 0x13, 0x12, 0x02];

const CRIO_PACKET_ID: u8 = 0x06;

const FLECS_UNITS: [&str; 16] = [
    "ppg", "ppg", "gpm", "bpm", "psi", "psi", "gal", "gal", "bbl", "bbl", "cmt%", "wtr%", "bpm",
    "bpm", "DigOut", "",
];

pub fn is_crio_frame(data: &[u8]) -> bool {
    const LLC_OFFSET: usize = 14;
    const DATA_OFFSET: usize = 18;

    if data.len() < DATA_OFFSET + CRIO_SIGNATURE.len() + 1 {
        return false;
    }

    let length = u16::from_be_bytes([data[12], data[13]]);

    if !(4..=1500).contains(&length) {
        return false;
    }

    if data.len() < LLC_OFFSET + usize::from(length) {
        return false;
    }

    if data[14] != 0x03 || data[15] != 0x03 {
        return false;
    }

    if data[16..18] != [0x00, 0x08] {
        return false;
    }

    let signature_end = DATA_OFFSET + CRIO_SIGNATURE.len();

    data[DATA_OFFSET..signature_end] == CRIO_SIGNATURE && data[signature_end] == CRIO_PACKET_ID
}

pub fn is_flecs_line(line: &str) -> bool {
    let fields: Vec<&str> = line.split(',').collect();

    if fields.len() < FLECS_UNITS.len() {
        return false;
    }

    fields
        .iter()
        .zip(FLECS_UNITS)
        .all(|(field, expected_unit)| {
            let mut parts = field.split_whitespace();

            let Some(value) = parts.next() else {
                return false;
            };

            let unit = parts.next().unwrap_or("");

            unit == expected_unit
                && parts.next().is_none()
                && value.parse::<f32>().is_ok_and(|number| number.is_finite())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLECS_SAMPLE: &str = concat!(
        "13.44 ppg,10.88 ppg,12 gpm,7.0 bpm,",
        "1359 psi,4413 psi,0 gal,0 gal,",
        "0.0 bbl,0.0 bbl,81.89 cmt%,",
        "82.55 wtr%,0.0 bpm,0.0 bpm,",
        "0 DigOut,0"
    );

    #[test]
    fn flecs_valid() {
        assert!(is_flecs_line(FLECS_SAMPLE));
    }

    #[test]
    fn flecs_invalid_unit() {
        assert!(!is_flecs_line(&FLECS_SAMPLE.replacen("ppg", "psi", 1)));
    }

    #[test]
    fn flecs_missing_fields() {
        assert!(!is_flecs_line("12 ppg,13 ppg"));
    }

    #[test]
    fn flecs_extra_fields() {
        assert!(is_flecs_line(&format!("{FLECS_SAMPLE},80 F")));
    }

    #[test]
    fn crio_valid() {
        let mut frame = vec![0u8; 14];

        // LLC header (3) + signature (9) + ID (1).
        frame[13] = 13;
        frame.extend_from_slice(&[0x03, 0x03, 0x00]);
        frame.extend_from_slice(&CRIO_SIGNATURE);
        frame.push(CRIO_PACKET_ID);

        assert!(is_crio_frame(&frame));
    }

    #[test]
    fn crio_invalid_id() {
        let mut frame = vec![0u8; 14];

        frame[13] = 13;
        frame.extend_from_slice(&[0x03, 0x03, 0x00]);
        frame.extend_from_slice(&CRIO_SIGNATURE);
        frame.push(0x09);

        assert!(!is_crio_frame(&frame));
    }
}
