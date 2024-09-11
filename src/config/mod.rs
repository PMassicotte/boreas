#![allow(dead_code)]
use chrono::{Duration, Months, NaiveDate};
use serde::de::Error;
use serde::Deserialize;
use serde::Deserializer;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod error;
use error::ConfigError;

mod timestep;
use timestep::TimeStep;

#[derive(Debug)]
pub struct Config {
    start_date: NaiveDate,
    end_date: NaiveDate,
    frequency: TimeStep,
    hourly_increment: u8,
}

// This function deserializes a Config object from a deserializer, ensuring the dates are valid and
// in order, and the hourly increment is within an acceptable range.
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ConfigHelper {
            start_date: String,
            end_date: String,
            frequency: TimeStep,
            hourly_increment: u8,
        }

        // Deserialize into the helper struct
        let helper = ConfigHelper::deserialize(deserializer)?;

        // Parse start_date
        let start_date = NaiveDate::parse_from_str(&helper.start_date, "%Y-%m-%d")
            .map_err(|e| D::Error::custom(format!("Invalid start_date format: {}", e)))?;

        // Parse end_date
        let end_date = NaiveDate::parse_from_str(&helper.end_date, "%Y-%m-%d")
            .map_err(|e| D::Error::custom(format!("Invalid end_date format: {}", e)))?;

        // Ensure start_date is before end_date
        if start_date > end_date {
            return Err(D::Error::custom(ConfigError::DateOrder));
        }

        // Validate hourly_increment
        let valid_timestep = [1, 2, 3, 4, 6, 8, 12];
        if !valid_timestep.contains(&helper.hourly_increment) {
            return Err(D::Error::custom(ConfigError::HourlyIncrement));
        }

        Ok(Config {
            start_date,
            end_date,
            frequency: helper.frequency,
            hourly_increment: helper.hourly_increment,
        })
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // TODO: how to validate that the start_date < end_date
        let config: Config = serde_json::from_reader(reader).map_err(ConfigError::from)?;

        Ok(config)
    }

    fn increment_date(&self, current_date: NaiveDate) -> Result<NaiveDate, String> {
        match self.frequency {
            TimeStep::Daily => Ok(current_date + Duration::days(1)),
            TimeStep::Weekly => Ok(current_date + Duration::weeks(1)),
            TimeStep::Monthly => current_date
                .checked_add_months(Months::new(1))
                .ok_or_else(|| format!("Failed to add a month to date: {}", current_date)),
        }
    }
}

impl Iterator for Config {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_date <= self.end_date {
            let current_date = self.start_date;
            self.start_date = self.increment_date(self.start_date).ok()?;
            Some(current_date)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_from_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let mut file = File::create(&file_path).unwrap();

        let config_data = r#"
    {
        "start_date": "2023-01-01",
        "end_date": "2023-01-10",
        "frequency": "daily",
        "hourly_increment": 3
    }
    "#;

        file.write_all(config_data.as_bytes()).unwrap();

        let config = Config::from_file(file_path).unwrap();

        assert_eq!(config.frequency, TimeStep::Daily);

        assert_eq!(
            config.start_date,
            NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date")
        );

        assert_eq!(
            config.end_date,
            NaiveDate::from_ymd_opt(2023, 1, 10).expect("Invalid date")
        );
    }

    #[test]
    fn test_increment_date_daily() {
        let config = Config {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 10).expect("Invalid date"),
            frequency: TimeStep::Daily,
            hourly_increment: 1,
        };

        let new_date = config
            .increment_date(NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"))
            .unwrap();

        assert_eq!(
            new_date,
            NaiveDate::from_ymd_opt(2023, 1, 2).expect("Invalid date")
        );
    }

    #[test]
    fn test_increment_date_weekly() {
        let config = Config {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 10).expect("Invalid date"),
            frequency: TimeStep::Weekly,
            hourly_increment: 1,
        };

        let new_date = config
            .increment_date(NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"))
            .unwrap();

        assert_eq!(
            new_date,
            NaiveDate::from_ymd_opt(2023, 1, 8).expect("Invalid date")
        );
    }

    #[test]
    fn test_increment_date_monthly() {
        let config = Config {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 31).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 12, 31).expect("Invalid date"),
            frequency: TimeStep::Monthly,
            hourly_increment: 1,
        };

        let new_date = config
            .increment_date(NaiveDate::from_ymd_opt(2023, 1, 31).expect("Invalid date"))
            .unwrap();

        assert_eq!(
            new_date,
            NaiveDate::from_ymd_opt(2023, 2, 28).expect("Invalid date")
        ); // February 31st is invalid, should fallback to 28th
    }

    #[test]
    fn test_iterator() {
        let config = Config {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 3).expect("Invalid date"),
            frequency: TimeStep::Daily,
            hourly_increment: 3,
        };

        let dates: Vec<NaiveDate> = config.collect();

        assert_eq!(
            dates,
            vec![
                NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
                NaiveDate::from_ymd_opt(2023, 1, 2).expect("Invalid date"),
                NaiveDate::from_ymd_opt(2023, 1, 3).expect("Invalid date"),
            ]
        );
    }
}
