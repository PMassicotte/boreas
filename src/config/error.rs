use crate::config::timestep::TimeStepParseError;

use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    DateOrder,
    DateParse(chrono::ParseError),
    TimeStep(TimeStepParseError),
    Io(std::io::Error),
    Json(serde_json::Error),
    HourlyIncrement,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::DateOrder => write!(f, "end_date cannot be earlier than start_date"),
            ConfigError::DateParse(e) => write!(f, "Failed to parse date: {}", e),
            ConfigError::TimeStep(e) => write!(f, "{}", e),
            ConfigError::Io(e) => write!(f, "I/O error: {}", e),
            ConfigError::Json(e) => write!(f, "Failed to parse JSON: {}", e),
            ConfigError::HourlyIncrement => {
                write!(f, "hourly_increment should one of 1, 2, 3, 4, 6, 8, 12")
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> ConfigError {
        ConfigError::Io(err)
    }
}

impl From<chrono::ParseError> for ConfigError {
    fn from(err: chrono::ParseError) -> ConfigError {
        ConfigError::DateParse(err)
    }
}

impl From<TimeStepParseError> for ConfigError {
    fn from(err: TimeStepParseError) -> ConfigError {
        ConfigError::TimeStep(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> ConfigError {
        ConfigError::Json(err)
    }
}
