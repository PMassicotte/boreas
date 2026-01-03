use crate::error::Result;
use gdal::Dataset;
use std::collections::HashMap;

/// Type-safe enumeration of oceanographic datasets
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum DatasetType {
    Chlorophyll,                       // chl_a - Chlorophyll-a concentration
    SeaSurfaceTemperature,             // sst - Sea surface temperature
    PhotosyntheticallyActiveRadiation, // par - Photosynthetically active radiation
    Kd490,                             // kd_490 - Diffuse attenuation coefficient at 490nm
    Rrs443,                            // rrs_443 - Remote sensing reflectance at 443nm
    Rrs488,                            // rrs_488 - Remote sensing reflectance at 488nm
    Rrs555,                            // rrs_555 - Remote sensing reflectance at 555nm
}

impl DatasetType {
    /// Convert from config name to DatasetType
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "chl_a" | "chlor_a" | "chlorophyll" => Some(Self::Chlorophyll),
            "sst" | "sea_surface_temperature" => Some(Self::SeaSurfaceTemperature),
            "par" | "photosynthetically_active_radiation" => {
                Some(Self::PhotosyntheticallyActiveRadiation)
            }
            "kd_490" | "kd490" => Some(Self::Kd490),
            "rrs_443" | "rrs443" => Some(Self::Rrs443),
            "rrs_488" | "rrs488" => Some(Self::Rrs488),
            "rrs_555" | "rrs555" => Some(Self::Rrs555),
            _ => None,
        }
    }

    /// Get the standard config name for this dataset type
    #[allow(dead_code)]
    pub fn config_name(&self) -> &str {
        match self {
            Self::Chlorophyll => "chl_a",
            Self::SeaSurfaceTemperature => "sst",
            Self::PhotosyntheticallyActiveRadiation => "par",
            Self::Kd490 => "kd_490",
            Self::Rrs443 => "rrs_443",
            Self::Rrs488 => "rrs_488",
            Self::Rrs555 => "rrs_555",
        }
    }
}

pub trait PrimaryProduction {
    /// Calculate primary production for a specified region
    ///
    /// # Arguments
    /// * `datasets` - HashMap of datasets by type
    /// * `x_start` - Starting x coordinate (column)
    /// * `y_start` - Starting y coordinate (row)
    /// * `width` - Width of the region to process
    /// * `height` - Height of the region to process
    fn calculate(
        &self,
        datasets: &HashMap<DatasetType, Dataset>,
        x_start: u32,
        y_start: u32,
        width: u32,
        height: u32,
    ) -> Result<Vec<f64>>;

    /// Get the name of this algorithm
    fn name(&self) -> &str;

    /// Get the list of required datasets for this algorithm
    #[allow(dead_code)]
    fn required_datasets(&self) -> Vec<DatasetType>;
}
