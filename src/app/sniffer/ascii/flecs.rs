#[derive(Debug, PartialEq)]
pub struct FlecsData {
    pub recirc_density: f32,
    pub downhole_density: f32,
    pub mix_water_rate: f32,
    pub combo_rate: f32,
    pub ps_pressure: f32,
    pub ds_pressure: f32,
    pub mix_wtr_stg_ttl: f32,
    pub mix_wtr_job_ttl: f32,
    pub combo_pump_stg_ttl: f32,
    pub combo_pump_job_ttl: f32,
    pub cement_vlv_percent: f32,
    pub wtr_vlv_percent: f32,
    pub ps_rate: f32,
    pub ds_rate: f32,
}

const UNITS: [&str; 14] = [
    "ppg", "ppg", "gpm", "bpm", "psi", "psi", "gal", "gal", "bbl", "bbl", "cmt%", "wtr%", "bpm",
    "bpm",
];

pub fn parse_line(line: &str) -> Result<FlecsData, String> {
    let fields: Vec<&str> = line.split(',').collect();

    if fields.len() < UNITS.len() {
        return Err(format!("Недостаточно полей: {}", fields.len()));
    }

    let mut values = [0.0_f32; 14];

    for (index, expected_unit) in UNITS.iter().enumerate() {
        let mut parts = fields[index].split_whitespace();

        let raw = parts.next().ok_or_else(|| format!("Пустое поле {index}"))?;

        let actual_unit = parts.next().unwrap_or("");

        if actual_unit != *expected_unit || parts.next().is_some() {
            return Err(format!("Неверная единица измерения поля {index}"));
        }

        let value = raw
            .parse::<f32>()
            .map_err(|_| format!("Неверное число в поле {index}"))?;

        if !value.is_finite() {
            return Err(format!("Неконечное число в поле {index}"));
        }

        values[index] = value;
    }

    Ok(FlecsData {
        recirc_density: values[0],
        downhole_density: values[1],
        mix_water_rate: values[2],
        combo_rate: values[3],
        ps_pressure: values[4],
        ds_pressure: values[5],
        mix_wtr_stg_ttl: values[6],
        mix_wtr_job_ttl: values[7],
        combo_pump_stg_ttl: values[8],
        combo_pump_job_ttl: values[9],
        cement_vlv_percent: values[10],
        wtr_vlv_percent: values[11],
        ps_rate: values[12],
        ds_rate: values[13],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        "13.44 ppg,10.88 ppg,12 gpm,7.0 bpm,",
        "1359 psi,4413 psi,0 gal,0 gal,",
        "0.0 bbl,0.0 bbl,81.89 cmt%,82.55 wtr%,",
        "0.0 bpm,0.0 bpm,0 DigOut,0"
    );

    #[test]
    fn parses_fourteen_fields() {
        let data = parse_line(SAMPLE).unwrap();

        assert_eq!(data.recirc_density, 13.44);
        assert_eq!(data.cement_vlv_percent, 81.89);
        assert_eq!(data.wtr_vlv_percent, 82.55);
        assert_eq!(data.ds_rate, 0.0);
    }

    #[test]
    fn rejects_wrong_unit() {
        let line = SAMPLE.replacen("ppg", "psi", 1);
        assert!(parse_line(&line).is_err());
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(parse_line("13.44 ppg,10.88 ppg").is_err());
    }
}
