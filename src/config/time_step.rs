use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum TimeStep {
    #[serde(rename(deserialize = "daily"))]
    Daily,
    #[serde(rename(deserialize = "weekly"))]
    Weekly,
    #[serde(rename(deserialize = "monthly"))]
    Monthly,
}

#[derive(Debug)]
pub struct TimeStepParseError;

impl fmt::Display for TimeStepParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid time step")
    }
}

impl std::error::Error for TimeStepParseError {}
