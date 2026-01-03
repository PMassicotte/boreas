use thiserror::Error;

#[derive(Error, Debug)]
pub enum BoreasError {
    #[error("GDAL error: {0}")]
    Gdal(#[from] gdal::errors::GdalError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Invalid dimensions: {0}")]
    DimensionMismatch(String),

    #[error("Missing dataset: {0}")]
    MissingDataset(String),

    #[error("Calculation error in {model}: {reason}")]
    Calculation { model: String, reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BoreasError>;
