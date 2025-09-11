use crate::config::Config;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

#[allow(dead_code)]
pub struct DateTimeGenerator {
    config: Config,
}

impl DateTimeGenerator {
    #[allow(dead_code)]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    #[allow(dead_code)]
    pub fn generate_datetime_series(&self) -> Vec<NaiveDateTime> {
        let hourly_increment = self.config.hourly_increment();
        if hourly_increment == 0 {
            eprintln!("Error: hourly_increment must be greater than 0 to avoid division by zero.");
            return Vec::new();
        }

        let mut datetimes = Vec::new();

        // Clone config to use as iterator
        let config_iter = self.config.clone();

        for date in config_iter {
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
        let config_iter = self.config.clone();

        config_iter.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TimeStep;
    use chrono::{NaiveDate, Timelike};

    fn create_test_config() -> Config {
        Config::new(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
            TimeStep::Daily,
            6, // Every 6 hours
        )
    }

    #[test]
    fn test_generate_datetime_series() {
        let config = create_test_config();
        let generator = DateTimeGenerator::new(config);
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
