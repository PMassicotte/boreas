use crate::config::Config;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

#[allow(dead_code)]
pub struct DateTimeGenerator<'a> {
    config: &'a Config,
}

impl<'a> DateTimeGenerator<'a> {
    #[allow(dead_code)]
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    #[allow(dead_code)]
    pub fn generate_datetime_series(&self) -> Vec<NaiveDateTime> {
        if self.config.hourly_increment() == 0 {
            eprintln!("Error: hourly_increment must be greater than 0 to avoid division by zero.");
            return Vec::new();
        }

        let mut datetimes = Vec::new();

        for date in self.config.dates() {
            let hours_in_day = 24 / self.config.hourly_increment() as u32;

            for hour_step in 0..hours_in_day {
                let hour = hour_step * self.config.hourly_increment() as u32;
                let datetime = date
                    .and_hms_opt(hour, 0, 0)
                    .unwrap_or_else(|| date.and_hms_opt(0, 0, 0).unwrap());
                datetimes.push(datetime);
            }
        }

        datetimes
    }

    #[allow(dead_code)]
    pub fn generate_date_series(&self) -> Vec<NaiveDate> {
        self.config.dates().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Timelike};
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_config() -> Config {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.json");
        let mut file = File::create(&file_path).unwrap();

        let config_data = r#"
        {
            "model_id": "Test",
            "start_date": "2023-01-01",
            "end_date": "2023-01-02",
            "frequency": "daily",
            "hourly_increment": 6,
            "raster_templates": [],
            "aoi": {
                "type": "bbox",
                "xmin": 0.0,
                "xmax": 1.0,
                "ymin": 0.0,
                "ymax": 1.0
            },
            "output_directory": "/tmp",
            "algorithm": "vgpm"
        }
        "#;

        file.write_all(config_data.as_bytes()).unwrap();
        Config::from_file(file_path).unwrap()
    }

    #[test]
    fn test_generate_datetime_series() {
        let config = create_test_config();
        let generator = DateTimeGenerator::new(&config);
        let series = generator.generate_datetime_series();

        // Should have 2 days * 4 time points per day (every 6 hours)
        assert_eq!(series.len(), 8);

        // Check first day times
        assert_eq!(series[0].hour(), 0);
        assert_eq!(series[1].hour(), 6);
        assert_eq!(series[2].hour(), 12);
        assert_eq!(series[3].hour(), 18);

        // Check second day
        assert_eq!(
            series[4].date(),
            NaiveDate::from_ymd_opt(2023, 1, 2).unwrap()
        );
    }
}
