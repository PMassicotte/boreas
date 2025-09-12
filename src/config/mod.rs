#![allow(dead_code)]
use chrono::{Duration, Months, NaiveDate};

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::bbox::Bbox;

pub mod error;
pub use error::ConfigError;

pub mod timestep;
pub use timestep::TimeStep;

#[derive(Debug, Deserialize, Clone)]
pub struct RasterFile {
    pub name: String,
    pub base_directory: String,
    pub filename_pattern: String,
    pub date_format: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    model_id: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    frequency: TimeStep,
    hourly_increment: u8,
    bbox: Bbox,
    raster_templates: Vec<RasterFile>,
    output_directory: String,
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
            model_id: Option<String>,
            start_date: String,
            end_date: String,
            frequency: TimeStep,
            hourly_increment: u8,
            raster_templates: Option<Vec<RasterFile>>,
            bbox: Option<BboxHelper>,
            output_directory: Option<String>,
        }

        #[derive(Deserialize)]
        struct BboxHelper {
            xmin: f64,
            xmax: f64,
            ymin: f64,
            ymax: f64,
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

        // Validate model_id is required and not empty
        let model_id = helper
            .model_id
            .ok_or_else(|| D::Error::custom("model_id is required"))?;
        if model_id.trim().is_empty() {
            return Err(D::Error::custom("model_id cannot be empty"));
        }

        // Validate raster_templates is required and validate each template
        let raster_templates = helper
            .raster_templates
            .ok_or_else(|| D::Error::custom("raster_templates is required"))?;

        // Validate each raster template
        for template in &raster_templates {
            if template.name.trim().is_empty() {
                return Err(D::Error::custom("raster template name cannot be empty"));
            }
            if template.base_directory.trim().is_empty() {
                return Err(D::Error::custom(
                    "raster template base_directory cannot be empty",
                ));
            }
            if template.filename_pattern.trim().is_empty() {
                return Err(D::Error::custom(
                    "raster template filename_pattern cannot be empty",
                ));
            }
            if template.date_format.trim().is_empty() {
                return Err(D::Error::custom(
                    "raster template date_format cannot be empty",
                ));
            }
            if !template.filename_pattern.contains("{}") {
                return Err(D::Error::custom(
                    "raster template filename_pattern must contain '{}' placeholder",
                ));
            }
        }

        // Validate bbox is required
        let bbox = helper
            .bbox
            .ok_or_else(|| D::Error::custom("bbox is required"))?;
        let bbox = Bbox::new(bbox.xmin, bbox.xmax, bbox.ymin, bbox.ymax)
            .map_err(|e| D::Error::custom(format!("Invalid bbox: {}", e)))?;

        // Validate output directory is required
        let output_directory = helper
            .output_directory
            .ok_or_else(|| D::Error::custom("output_directory is required"))?;
        if !Path::new(&output_directory).exists() {
            return Err(D::Error::custom(ConfigError::OutputDirectory(
                output_directory.clone(),
            )));
        }

        Ok(Config {
            model_id,
            start_date,
            end_date,
            frequency: helper.frequency,
            hourly_increment: helper.hourly_increment,
            raster_templates,
            bbox,
            output_directory,
        })
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let config: Config = serde_json::from_reader(reader).map_err(ConfigError::from)?;

        Ok(config)
    }

    pub fn hourly_increment(&self) -> u8 {
        self.hourly_increment
    }

    pub fn raster_templates(&self) -> &Vec<RasterFile> {
        &self.raster_templates
    }

    pub fn bbox(&self) -> &Bbox {
        &self.bbox
    }

    pub fn output_directory(&self) -> &String {
        &self.output_directory
    }

    pub fn model_id(&self) -> &String {
        &self.model_id
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
        "model_id": "test_model",
        "start_date": "2023-01-01",
        "end_date": "2023-01-10",
        "frequency": "daily",
        "hourly_increment": 3,
        "raster_templates": [],
        "bbox": {
            "xmin": 0.0,
            "xmax": 1.0,
            "ymin": 0.0,
            "ymax": 1.0
        },
        "output_directory": "/tmp"
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
            model_id: "test_model".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 10).expect("Invalid date"),
            frequency: TimeStep::Daily,
            hourly_increment: 1,
            raster_templates: vec![],
            bbox: Bbox::new(0.0, 1.0, 0.0, 1.0).unwrap(),
            output_directory: "/tmp".to_string(),
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
            model_id: "test_model".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 10).expect("Invalid date"),
            frequency: TimeStep::Weekly,
            hourly_increment: 1,
            raster_templates: vec![],
            bbox: Bbox::new(0.0, 1.0, 0.0, 1.0).unwrap(),
            output_directory: "/tmp".to_string(),
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
            model_id: "test_model".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 31).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 12, 31).expect("Invalid date"),
            frequency: TimeStep::Monthly,
            hourly_increment: 1,
            raster_templates: vec![],
            bbox: Bbox::new(0.0, 1.0, 0.0, 1.0).unwrap(),
            output_directory: "/tmp".to_string(),
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
            model_id: "test_model".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).expect("Invalid date"),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 3).expect("Invalid date"),
            frequency: TimeStep::Daily,
            hourly_increment: 3,
            raster_templates: vec![],
            bbox: Bbox::new(0.0, 1.0, 0.0, 1.0).unwrap(),
            output_directory: "/tmp".to_string(),
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
