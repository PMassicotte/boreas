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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CbpmModel, VgpmModel};

    #[test]
    fn test_dataset_type_from_name_valid() {
        // Test chlorophyll aliases
        assert_eq!(
            DatasetType::from_name("chl_a"),
            Some(DatasetType::Chlorophyll)
        );
        assert_eq!(
            DatasetType::from_name("chlor_a"),
            Some(DatasetType::Chlorophyll)
        );
        assert_eq!(
            DatasetType::from_name("chlorophyll"),
            Some(DatasetType::Chlorophyll)
        );

        // Test SST aliases
        assert_eq!(
            DatasetType::from_name("sst"),
            Some(DatasetType::SeaSurfaceTemperature)
        );
        assert_eq!(
            DatasetType::from_name("sea_surface_temperature"),
            Some(DatasetType::SeaSurfaceTemperature)
        );

        // Test PAR aliases
        assert_eq!(
            DatasetType::from_name("par"),
            Some(DatasetType::PhotosyntheticallyActiveRadiation)
        );
        assert_eq!(
            DatasetType::from_name("photosynthetically_active_radiation"),
            Some(DatasetType::PhotosyntheticallyActiveRadiation)
        );

        // Test Kd490 aliases
        assert_eq!(DatasetType::from_name("kd_490"), Some(DatasetType::Kd490));
        assert_eq!(DatasetType::from_name("kd490"), Some(DatasetType::Kd490));

        // Test Rrs443 aliases
        assert_eq!(DatasetType::from_name("rrs_443"), Some(DatasetType::Rrs443));
        assert_eq!(DatasetType::from_name("rrs443"), Some(DatasetType::Rrs443));

        // Test Rrs488 aliases
        assert_eq!(DatasetType::from_name("rrs_488"), Some(DatasetType::Rrs488));
        assert_eq!(DatasetType::from_name("rrs488"), Some(DatasetType::Rrs488));

        // Test Rrs555 aliases
        assert_eq!(DatasetType::from_name("rrs_555"), Some(DatasetType::Rrs555));
        assert_eq!(DatasetType::from_name("rrs555"), Some(DatasetType::Rrs555));
    }

    #[test]
    fn test_dataset_type_from_name_case_insensitive() {
        // Test case insensitivity
        assert_eq!(
            DatasetType::from_name("CHL_A"),
            Some(DatasetType::Chlorophyll)
        );
        assert_eq!(
            DatasetType::from_name("SST"),
            Some(DatasetType::SeaSurfaceTemperature)
        );
        assert_eq!(
            DatasetType::from_name("PAR"),
            Some(DatasetType::PhotosyntheticallyActiveRadiation)
        );
        assert_eq!(
            DatasetType::from_name("KD_490"),
            Some(DatasetType::Kd490)
        );
        assert_eq!(
            DatasetType::from_name("RRS_443"),
            Some(DatasetType::Rrs443)
        );
    }

    #[test]
    fn test_dataset_type_from_name_invalid() {
        // Test invalid names
        assert_eq!(DatasetType::from_name("invalid"), None);
        assert_eq!(DatasetType::from_name(""), None);
        assert_eq!(DatasetType::from_name("chl"), None);
        assert_eq!(DatasetType::from_name("temperature"), None);
        assert_eq!(DatasetType::from_name("rrs"), None);
    }

    #[test]
    fn test_dataset_type_config_name() {
        // Test that config_name returns the expected standard names
        assert_eq!(DatasetType::Chlorophyll.config_name(), "chl_a");
        assert_eq!(
            DatasetType::SeaSurfaceTemperature.config_name(),
            "sst"
        );
        assert_eq!(
            DatasetType::PhotosyntheticallyActiveRadiation.config_name(),
            "par"
        );
        assert_eq!(DatasetType::Kd490.config_name(), "kd_490");
        assert_eq!(DatasetType::Rrs443.config_name(), "rrs_443");
        assert_eq!(DatasetType::Rrs488.config_name(), "rrs_488");
        assert_eq!(DatasetType::Rrs555.config_name(), "rrs_555");
    }

    #[test]
    fn test_dataset_type_from_name_config_name_consistency() {
        // Test that config_name can be parsed back by from_name
        let all_types = vec![
            DatasetType::Chlorophyll,
            DatasetType::SeaSurfaceTemperature,
            DatasetType::PhotosyntheticallyActiveRadiation,
            DatasetType::Kd490,
            DatasetType::Rrs443,
            DatasetType::Rrs488,
            DatasetType::Rrs555,
        ];

        for dataset_type in all_types {
            let config_name = dataset_type.config_name();
            let parsed = DatasetType::from_name(config_name);
            assert_eq!(
                parsed,
                Some(dataset_type),
                "Failed for config_name: {}",
                config_name
            );
        }
    }

    #[test]
    fn test_vgpm_required_datasets() {
        let model = VgpmModel::new();
        let required = model.required_datasets();

        // VGPM should require exactly 3 datasets
        assert_eq!(required.len(), 3);

        // Check that it includes the expected datasets
        assert!(required.contains(&DatasetType::Chlorophyll));
        assert!(required.contains(&DatasetType::SeaSurfaceTemperature));
        assert!(required.contains(&DatasetType::Kd490));
    }

    #[test]
    fn test_cbpm_required_datasets() {
        let model = CbpmModel::new();
        let required = model.required_datasets();

        // CbPM should require exactly 3 datasets
        assert_eq!(required.len(), 3);

        // Check that it includes the expected datasets
        assert!(required.contains(&DatasetType::Chlorophyll));
        assert!(required.contains(&DatasetType::SeaSurfaceTemperature));
        assert!(required.contains(&DatasetType::Kd490));
    }

    #[test]
    fn test_vgpm_model_name() {
        let model = VgpmModel::new();
        assert_eq!(model.name(), "VGPM");
    }

    #[test]
    fn test_cbpm_model_name() {
        let model = CbpmModel::new();
        assert_eq!(model.name(), "CbPM");
    }
}
